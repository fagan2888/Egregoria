use crate::{BuildingID, IntersectionID, LotID, RoadID};
use flat_spatial::shapegrid::ShapeGridHandle;
use flat_spatial::ShapeGrid;
use geom::{Circle, Vec2, AABB};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    Building(BuildingID),
    Lot(LotID),
    Ground,
}

macro_rules! impl_from_pk {
    ($t: ty, $e: expr) => {
        impl From<$t> for ProjectKind {
            fn from(x: $t) -> Self {
                $e(x)
            }
        }
    };
}

impl_from_pk!(IntersectionID, ProjectKind::Inter);
impl_from_pk!(RoadID, ProjectKind::Road);
impl_from_pk!(BuildingID, ProjectKind::Building);
impl_from_pk!(LotID, ProjectKind::Lot);

impl ProjectKind {
    pub fn to_lot(self) -> Option<LotID> {
        if let ProjectKind::Lot(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

pub struct SpatialMap {
    grid: ShapeGrid<ProjectKind, AABB>,
    ids: HashMap<ProjectKind, ShapeGridHandle>,
}

impl Default for SpatialMap {
    fn default() -> Self {
        Self {
            grid: ShapeGrid::new(50),
            ids: Default::default(),
        }
    }
}

impl SpatialMap {
    pub fn insert<T: Into<ProjectKind>>(&mut self, p: T, bbox: AABB) {
        let kind = p.into();
        let handle = self.grid.insert(bbox, kind);
        self.ids.insert(kind, handle);
    }

    pub fn remove<T: Into<ProjectKind>>(&mut self, p: T) {
        let kind = p.into();
        if let Some(id) = self.ids.remove(&kind) {
            self.grid.remove(id);
        } else {
            warn!(
                "trying to remove {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn update<T: Into<ProjectKind>>(&mut self, p: T, bbox: AABB) {
        let kind = p.into();
        if let Some(id) = self.ids.get(&kind) {
            self.grid.set_shape(*id, bbox);
        } else {
            warn!(
                "trying to update shape {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn query_around(
        &self,
        center: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = ProjectKind> + '_ {
        self.grid
            .query(Circle {
                center: center.into(),
                radius,
            })
            .map(|(_, _, k)| *k)
    }

    pub fn query_rect(&self, r: AABB) -> impl Iterator<Item = ProjectKind> + '_ {
        self.grid.query(r).map(|(_, _, k)| *k)
    }

    pub fn query_point(&self, p: Vec2) -> impl Iterator<Item = ProjectKind> + '_ {
        self.grid.query([p.x, p.y]).map(|(_, _, k)| *k)
    }

    pub fn debug_grid(&self) -> impl Iterator<Item = AABB> + '_ {
        self.grid
            .handles()
            .filter_map(move |x| self.grid.get(x))
            .map(|(aabb, _)| *aabb)
    }
}
