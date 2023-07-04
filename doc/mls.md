# Implementing a reasonable MLS for IRC/Sable

## Foreword: About "Delivery Service" (DS) and "Authentication Service" (AS)

In MLS, two important entities are DS: Delivery Service and AS: Authentication Service.

DS is supposed not trusted though it can hold power and mount some attacks (section 16.XX of MLS spec).

AS is supposed trusted because it is the source of truth regarding identities, e.g. "this person is indeed X"
and can therefore manipulate private group memberships (see section 16.XX for compromises and mitigations).

In this context, IRCd usually offers strong authentication via SASL, therefore AS is either the IRCd or the SASL backend directly
and must be trusted.

Finally, DS is necessarily the IRCd active connection which delivers various messages.

**Conclusion** :

MLS in IRC assumes that IRCd as a DS and AS **is trusted**, except in presence of external SASL backend separate from the IRCd, with a different
trust domain.

It is not a problem because the proposal shines when considered in interaction with bouncers and multi-tenant clients such as IRCCloud, The Lounge, etc, where client-side "encryption keys" will achieve the same level of confidence as in Matrix.

## Capabilities

**Dependencies**:

- `sasl`: for AS
- `metadata-2`: <https://github.com/ircv3/ircv3-specifications/pull/501/files>

As MLS is a complicated beast, we will offer increasingly powerful capabilities to achieve ideal support of MLS in IRC.

On server-side:

- `sable/mls/keypackage` : offers a way for user to publish their `KeyPackage` and let other users retrieve them for client-mediated MLS
- `sable/mls/channel` : offers a way for user to send directly MLS messages in a channel or over a query
- `sable/mls/public` : offers public channels with MLS protocol (i.e. external joins)
- `sable/mls/external-proposals` : the DS will perform external proposals based on AS policies to actively remove users (expired/revoked) or update all channels
- `sable/mls/standardreplies` : uses standard replies for communications

On client-side:

- `sable/mls/private` : client supports MLS protocol in "private" channels, including 1:1 chats
- `sable/mls/public` : client supports MLS protocol in "public" channels

**Note** : 

- `channel-rename` is incompatible with MLS-enabled channels until `sable/mls/rename` is implemented.
- 

## `sable/mls/keypackage`

**Summary** : Distribution of `KeyPackage` via DS using `metadata-2` offline and online for an authenticated user via `sasl`.

**Dependencies**:

- `sasl`: for AS
- `metadata-2`: for storing the `KeyPackage`

`mls/keyPackage` can never be set by any user on the IRCd, only the IRCd system can set those in response to user requests such as:

`METADATA * SET keyPackage $encodedKeyPackage`

A user can hold only one `keyPackage`, multi-device support is out of scope for this proposal.

Any user can retrieve the `keyPackage` of another user or the "combined" `keyPackage` of a channel formed by all its current user by:

`METADATA <user | channel> GET keyPackage`

## `sable/mls/channel`

**Summary** : Support of MLS messages in a channel, supposedly private.

**Dependencies** :

- [`draft/named-modes`](https://github.com/ircv3/ircv3-specifications/pull/484): for "mls" modes in a channel
- `sable/mls/keypackage` : for users can retrieve and build the adequate cryptographic structures
- `message-tags`
- `draft/multiline`: for text messages wrapped
- `draft/batch`: for multiline

### Backward compatibility considerations and active loss of MLS capability

Clients not supporting `sable/mls` will be barred any entry into a MLS-enabled channel with an
error message, if `standard-replies` is supported, it will be used.

The active loss of MLS capabilitiy does not endanger group membership as long as the MLS parameters
are safely stored by the client and they are not removed by the current users.

### Creating a MLS channel

Creating a MLS channel is the same as creating a new IRC channel, except
that you need to have `+sable/mls` to this channel.

This mode requires:

- `keyPackage` metadata attached to the channel to not be empty
- all current users in the channel supports `sable/mls`

Otherwise, you will receive an error.

When this mode is broadcasted, all clients present in the channel **MUST** prepare for MLS communications from now on.

The IRCd will choose a user^[This can be the user who sets the mode, but also a priority system can be devised to select in priority IRC ops, channel ops, half-ops, voice, normal users, this can be further restrained in the future.] who **have to** :

- initialize a one-member group, selecting an adequate group ID
- create a bunch of Add for every other user, fetching all keyPackage of the other members
- sends Welcome message to every other user once Commit is done.

Every client receiving a **Welcome** message **MUST** acknowledge it to the IRCd.

A client **SHOULD** not start sending encrypted messages before receiving a confirmation from IRCd that group is initialized, otherwise message loss is possible.

In case of when a client has not received any Welcome message after some timeout, e.g. 60 seconds, client **SHOULD** ask the IRCd to re-initialize the group by selecting someone else to re-encrypt the group.

**Note** : Service accounts can stay in a `+sable/mls` channel but will not be able to process
any message for now. A future capability such as `sable/mls/service-accounts` can enable
the capability of service accounts supporting MLS messages and participating into the E2EE.

### Joining a MLS channel

Joining an existing MLS channel will require the client supports `+sable/mls` and the current group parameters for ciphersuite and extensions.

If IRCd lets the user in, IRCd will produce an **external proposal** to `Add` this user and will wait for `Commit` acknowledgement of this proposal before reporting to the original user.

**Note** : In this capability, leaving a channel temporarily make you lose the right to read the history even if you can get via `CHATHISTORY`, this will be solved in `sable/mls/resume` for resumptions by prior group memberships.

### Inviting a user in a MLS channel

Inviting a user in an existing MLS channel works in two ways:

- inviting at the IRC level which induces a permission check
- inviting at the MLS level which induces a proposal commitment of adding the `keyPackage` of that specific user
- sends a Welcome message to the newly invited user when he joins

### Leaving a MLS channel

Leaving a MLS channel sends a Proposal to Remove yourself from the group which you can commit.

If the left user does not sends the removal proposal, any user or the server can submit an (external) proposal to remove the left user.

**Note** : Even in presence of `CHATHISTORY`, leaving has the same consequences as without `CHATHISTORY`, that is, loss of the backlog between disconnection periods. To restore "history" even with end to end encryption, it is necessary to devise a form of persistent session.

### Chatting in a MLS channel

Chatting in a MLS channel should reuse existing `PRIVMSG` / `NOTICE` commands.

The difference is that the parameter of `PRIVMSG` / `NOTICE` should be a [MLS Message](https://docs.rs/openmls/latest/openmls/framing/struct.MlsMessageOut.html) containing the usual plaintext payload.

**Example**:

Due to padding constraints and length issues, chatting in a MLS channel is much more pleasant in the presence of multi-line and batch features.

### Operating on a MLS channel

A MLS channel is still a IRC channel and is subject to moderation and administration.

Network services which do not depend upon the contents of message, e.g. automatic moderation via inspection of shady links, can work out of the box.

**Kicking** a user should send an external proposal to remove the user from the group.

Deleting / revoking a user from the authentication service should send an external proposal to all channels the user was potentially in to remove the user from the group.

**Note** : System messages to the channel, e.g. notices, cannot be encrypted, except in the potential upcoming `sable/mls/service-account` capability.

## `sable/mls/private`

**Summary** : Enable end-to-end encryption on a private channel, i.e. `+p`, `+s` or over a query.

Advertising this client capability means you have support for:

- `sasl`: for AS
- [`draft/metadata-2`](https://github.com/ircv3/ircv3-specifications/pull/501): for publishing `KeyPackage`
- [`draft/named-modes`](https://github.com/ircv3/ircv3-specifications/pull/484): for `+sable/mls` modes in a channel
- `sable/mls/keypackage` : for users can retrieve and build the adequate cryptographic structures
- `message-tags`
- `draft/multiline`: for text messages wrapped
- `draft/batch`: for multiline

### Publishing your `KeyPackage`

### Creating a MLS channel

### Joining/Leaving a MLS channel

### Inviting a MLS channel

### Chatting with a MLS channel
