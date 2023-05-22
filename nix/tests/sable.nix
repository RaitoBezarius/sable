{ pkgs, sableModule }:

let
  inherit (pkgs) lib system;

  deriveFingerprint = certFile: lib.removeSuffix "\n" (builtins.readFile (pkgs.runCommandLocal "openssl-get-sha1-fp" {} ''
    ${lib.getExe pkgs.openssl} x509 -in ${certFile} -outform DER | ${pkgs.coreutils}/bin/sha1sum | cut -d' ' -f1 > $out
  ''));

  wildcardTestCerts = rec {
    caCert = ./ca/minica.pem;
    caKey = ./ca/minica-key.pem;
    caFingerprint = deriveFingerprint caCert;
    nodeCert = ./ca/_.sable-server.test/cert.pem;
    nodeKey = ./ca/_.sable-server.test/key.pem;
    nodeFingerprint = deriveFingerprint nodeCert;
    mgmtCert = ./ca/mgmt.test/cert.pem;
    mgmtKey = ./ca/mgmt.test/cert.key;
    mgmtFingerprint = deriveFingerprint mgmtCert;
  };
  nodeConfig = { serverId, serverName, managementHostPort ? 8080, serverIP, caFile, managementClientCaFile, keyFile, certFile
    , bootstrapped ? false, peers ? null, authorisedUsersForManagement ? [] }: { ... }:
    let myFingerprint = deriveFingerprint certFile;
    in {
      imports = [ 
        sableModule
      ];

      networking.interfaces.eth1.ipv4.addresses = [
        { address = serverIP; prefixLength = 24; }
      ];

      networking.firewall.allowedTCPPorts = [ 6667 6668 6697 8888 ];

      virtualisation.forwardPorts = [{
        host.port = managementHostPort;
        guest.port = 8888;
      }];

      services.sable = {
        enable = true;
        package = pkgs.sable-dev;
        network.bootstrap = if bootstrapped then
          (lib.importJSON ../../configs/network_config.json)
        else
          null;
        network.settings = {
          fanout = 1;
          ca_file = caFile;
          peers = if (peers != null) then
            peers
          else ([{
            name = serverName;
            address = "${serverIP}:6668";
            fingerprint = myFingerprint;
          }]);
        };
        server.settings = let
          keys = {
            key_file = keyFile;
            cert_file = certFile;
          };
        in {
          server_id = serverId;
          server_name = serverName;

          management = {
            address = "127.0.0.1:8888";
            client_ca = managementClientCaFile;
            authorised_fingerprints = authorisedUsersForManagement;
          };

          server.listeners = [
            { address = "${serverIP}:6667"; }
            {
              address = "${serverIP}:6697";
              tls = true;
            }
          ];

          tls_config = keys;
          node_config = keys // { listen_addr = "${serverIP}:6668"; };
        };
      };
    };

  mkMultiNodeTest = { name, servers ? { }, client ? { }, testScript }:
    pkgs.nixosTest {
      inherit name testScript;
      nodes = servers // {
        client = { lib, ... }: {
        imports = [ sableModule client ];

        systemd.services.weechat-headless = {
          serviceConfig.StateDirectory = "weechat";
          script = ''
            ${pkgs.weechat}/bin/weechat-headless --stdout -d /var/lib/weechat --run "/logger set 9; /set fifo.file.path /tmp/weechat_fifo; /plugin unload fifo; /plugin load fifo; /fifo enable; /logger set 9"'';
          wantedBy = [ "multi-user.target" ];
          wants = [ "sable-ircd.service" ];
          after = [ "sable-ircd.service" ];
        };
      };
    };
  };

  mkBasicTest = { name, machine ? { }, testScript }:
    pkgs.nixosTest {
      inherit name testScript;
      nodes.machine = { lib, ... }: {
        imports = [
          sableModule
          (nodeConfig {
            serverId = 1;
            serverName = "server1.test";
            bootstrapped = true;
            caFile = ../../configs/ca_cert.pem;
            managementClientCaFile = ../../configs/ca_cert.pem;
            keyFile = ../../configs/server1.key;
            certFile = ../../configs/server1.pem;
            authorisedUsersForManagement = [{
              name = "user1";
              fingerprint = "435bc6db9f22e84ba5d9652432154617c9509370";
            }];
          })
          machine
        ];

        systemd.services.weechat-headless = {
          serviceConfig.StateDirectory = "weechat";
          script = ''
            ${pkgs.weechat}/bin/weechat-headless --stdout -d /var/lib/weechat --run "/logger set 9; /set fifo.file.path /tmp/weechat_fifo; /plugin unload fifo; /plugin load fifo; /fifo enable; /logger set 9"'';
          wantedBy = [ "multi-user.target" ];
          wants = [ "sable-ircd.service" ];
          after = [ "sable-ircd.service" ];
        };
      };
    };
in {
  monoNode = mkBasicTest {
    name = "mononode-sable";
    testScript = ''
      machine.start()
      machine.wait_for_unit("sable-ircd.service")
      machine.wait_for_unit("weechat-headless.service")

      def remote_weechat(command: str):
        return machine.succeed(f"echo \"{command}\" > /tmp/weechat_fifo")

      print(machine.succeed("sleep 1"))
      remote_weechat(" */server add test-ircd-nossl 127.0.1.2/6667")
      remote_weechat(" */connect test-ircd-nossl")
      remote_weechat("irc.server.test-ircd-nossl */nick test")
      remote_weechat("irc.server.test-ircd-nossl */join #hello")
      remote_weechat("irc.test-ircd-nossl.#hello *Hello world!")
    '';
  };

  basicMultiNodes = 
  let
    # 192.168.1.1 is client.
    ipAddressFromId = id: "192.168.1.${toString (id + 1)}";
    mkPeer = id: {
      name = "${toString id}.sable-network.test";
      address = "${ipAddressFromId id}:6668";
      fingerprint = wildcardTestCerts.nodeFingerprint;
    };
    mkPeers = nbServers:
      map mkPeer (lib.range 1 nbServers);
    mkNode = peers: id: nodeConfig {
      serverId = id;
      serverName = "${toString id}.sable-network.test";
      serverIP = ipAddressFromId id;

      # server1 is bootstrapped.
      bootstrapped = if id == 1 then true else false;

      caFile = wildcardTestCerts.caCert;

      managementClientCaFile = wildcardTestCerts.caCert;
      managementHostPort = 8080 + id;
      authorisedUsersForManagement = [{
        name = "test";
        fingerprint = wildcardTestCerts.mgmtFingerprint;
      }];

      certFile = wildcardTestCerts.nodeCert;
      keyFile = wildcardTestCerts.nodeKey;

      inherit peers;
    };

    nbServers = 5;
    peers = mkPeers nbServers;
    mkNetworkNode = mkNode peers;

    servers = lib.listToAttrs
      (map (index: lib.nameValuePair "server${toString index}" (mkNetworkNode index)) (lib.range 1 nbServers));
    serverNames = builtins.attrNames servers;
  in
  mkMultiNodeTest {
    name = "basic-multinodes-sable";
    inherit servers;
    testScript = ''
      import json
      start_all()

      servers = [globals()[server_name] for server_name in [${lib.concatMapStringsSep ", " (name: "\"${name}\"") serverNames}]]

      for server in servers:
        server.wait_for_unit("sable-ircd.service")

      def remote_weechat(command: str):
        return client.succeed(f"echo \"{command}\" > /tmp/weechat_fifo")

      def test_server(name: str, server: str, nick: str = "test", channel_to_join: str = "hello"):
        remote_weechat(f" */server add {name} {server}/6667")
        remote_weechat(f" */connect {name}")
        remote_weechat(f"irc.server.{name} */nick {nick}")
        remote_weechat(f"irc.server.{name} */join #{channel_to_join}")

      def get_network_state(server):
        # For now, certificate expired, so -k is necessary.
        intermediate = json.loads(server.succeed("curl -k --capath ${wildcardTestCerts.caCert} --cert ${wildcardTestCerts.mgmtCert} --key ${wildcardTestCerts.mgmtKey} https://localhost:8888/dump-network"))
        # Fix up 'servers' which are HashMap<Id, Type>
        intermediate['servers'] = {k: v for (k, v) in intermediate['servers']}
        return intermediate

      for server in servers:
        server.wait_for_open_port(8888)

      states = []
      for server in servers:
        states.append(get_network_state(server))

      for idx, (state_a, state_b) in enumerate(zip(states, states[1:])):
        assert state_a == state_b, f"Network state between server {idx} and {idx + 1} has diverged"

      print(states[-1])
    '';
  };
}
