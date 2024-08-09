pub mod user_note;
pub mod zone_funds;
pub mod zone_state;

pub fn assert_is_zone_note(
    zone_meta: &common::ZoneMetadata,
    note: &cl::NoteWitness,
    state_roots: &common::StateRoots,
) {
    assert_eq!(state_roots.commit().0, note.state);
    assert_eq!(zone_meta.id(), state_roots.zone_id);
    assert_eq!(zone_meta.zone_vk, note.death_constraint);
    assert_eq!(zone_meta.unit, note.unit);
}
