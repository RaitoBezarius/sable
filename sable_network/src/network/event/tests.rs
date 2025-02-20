use crate::prelude::*;
use super::*;

fn build_event_ids(input: &[[i64;3]]) -> Vec<EventId>
{
    let mut ret = Vec::new();
    for v in input.into_iter()
    {
        ret.push(EventId::new(ServerId::new(v[0]), EpochId::new(v[1]), v[2]));
    }
    ret
}

fn clock_from(ids: &[EventId]) -> EventClock
{
    let mut ret = EventClock::new();
    for id in ids.into_iter()
    {
        ret.update_with_id(*id);
    }
    ret
}

#[test]
fn clock_comparison()
{
    let ids1 = build_event_ids(&[
        [1,1,1],
        [1,1,2],
        [1,1,3],
        [1,2,1],
    ]);
    let ids2 = build_event_ids(&[
        [2,1,1],
        [2,1,2],
        [2,1,3],
    ]);

    let clock1 = clock_from(&[ids1[0], ids2[0]]);
    let clock2 = clock_from(&[ids1[1], ids2[1]]);

    assert!(clock1 <= clock2);

    let clock3 = clock_from(&[ids1[1]]);
    let clock4 = clock_from(&[ids1[1], ids2[1]]);

    assert!(clock3 <= clock4);

    let clock5 = clock_from(&[ids1[2]]);
    let clock6 = clock_from(&[ids1[3]]);

    assert!(clock5 < clock6);
}