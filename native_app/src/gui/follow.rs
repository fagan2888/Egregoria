use legion::Entity;

#[derive(Copy, Clone, Default)]
pub struct FollowEntity(pub Option<Entity>);
