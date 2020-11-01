use crate::procgen::Trees;
use crate::{
    Building, BuildingID, BuildingKind, Intersection, IntersectionID, Lane, LaneID, LaneKind,
    LanePattern, Lot, LotID, ParkingSpotID, ParkingSpots, ProjectKind, Road, RoadID,
    RoadSegmentKind, SpatialMap,
};
use geom::Spline;
use geom::Vec2;
use ordered_float::OrderedFloat;
use rand::prelude::IteratorRandom;
use rand::Rng;
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;
pub type Buildings = DenseSlotMap<BuildingID, Building>;
pub type Lots = DenseSlotMap<LotID, Lot>;

#[derive(Debug, Clone, Copy)]
pub struct MapProject {
    pub pos: Vec2,
    pub kind: ProjectKind,
}

pub struct Map {
    pub(crate) roads: Roads,
    pub(crate) lanes: Lanes,
    pub(crate) intersections: Intersections,
    pub(crate) buildings: Buildings,
    pub(crate) lots: Lots,
    pub(crate) spatial_map: SpatialMap,
    pub trees: Trees,
    pub parking: ParkingSpots,
    pub dirty: bool,
}

impl Default for Map {
    fn default() -> Self {
        Self::empty()
    }
}

impl Map {
    pub fn empty() -> Self {
        Self {
            roads: Roads::default(),
            lanes: Lanes::default(),
            intersections: Intersections::default(),
            parking: ParkingSpots::default(),
            buildings: Buildings::default(),
            lots: Lots::default(),
            trees: Trees::default(),
            dirty: true,
            spatial_map: SpatialMap::default(),
        }
    }

    pub fn update_intersection(&mut self, id: IntersectionID, f: impl Fn(&mut Intersection)) {
        info!("update_intersection {:?}", id);
        let inter = unwrap_or!(self.intersections.get_mut(id), return);
        f(inter);

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
        self.dirty = true;
    }

    fn invalidate(&mut self, id: IntersectionID) {
        info!("invalidate {:?}", id);

        self.dirty = true;
        let inter = &mut self.intersections[id];
        inter.update_interface_radius(&mut self.roads);

        for x in inter.roads.clone() {
            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_interface_radius(&mut self.roads);

            let road = &mut self.roads[x];
            road.gen_pos(&self.intersections, &mut self.lanes, &mut self.parking);

            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_polygon(&self.roads);
        }

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
        inter.update_polygon(&self.roads);

        self.spatial_map.update(inter.id, inter.polygon.bbox());
    }

    pub fn add_intersection(&mut self, pos: Vec2) -> IntersectionID {
        info!("add_intersection {:?}", pos);
        self.dirty = true;
        Intersection::make(&mut self.intersections, &mut self.spatial_map, pos)
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        info!("remove_intersection {:?}", src);

        self.dirty = true;
        for road in self.intersections[src].roads.clone() {
            self.remove_road(road);
        }

        self.spatial_map.remove(src);
        self.intersections.remove(src);
    }

    pub fn remove_building(&mut self, b: BuildingID) -> Option<Building> {
        info!("remove_building {:?}", b);

        let b = self.buildings.remove(b);
        if let Some(b) = &b {
            self.spatial_map.remove(b.id)
        }
        self.dirty |= b.is_some();
        b
    }

    pub fn split_road(&mut self, id: RoadID, pos: Vec2) -> IntersectionID {
        info!("split_road {:?} {:?}", id, pos);

        let r = self
            .remove_road(id)
            .expect("Trying to split unexisting road");
        let id = self.add_intersection(pos);

        let pat = r.pattern();
        match r.segment {
            RoadSegmentKind::Straight => {
                self.connect(r.src, id, &pat, RoadSegmentKind::Straight);
                self.connect(id, r.dst, &pat, RoadSegmentKind::Straight);
            }
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => {
                let s = Spline {
                    from: r.src_point,
                    to: r.dst_point,
                    from_derivative,
                    to_derivative,
                };
                let t_approx = s.project_t(pos, 1.0);

                let (s_from, s_to) = s.split_at(t_approx);

                self.connect(
                    r.src,
                    id,
                    &pat,
                    RoadSegmentKind::Curved((s_from.from_derivative, s_from.to_derivative)),
                );
                self.connect(
                    id,
                    r.dst,
                    &pat,
                    RoadSegmentKind::Curved((s_to.from_derivative, s_to.to_derivative)),
                );
            }
        }

        id
    }

    // todo: remove in favor of connect(..., RoadSegmentKind::Straight)
    pub fn connect_straight(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: &LanePattern,
    ) -> RoadID {
        self.connect(src, dst, pattern, RoadSegmentKind::Straight)
    }

    pub fn connect(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: &LanePattern,
        segment: RoadSegmentKind,
    ) -> RoadID {
        info!("connect {:?} {:?} {:?} {:?}", src, dst, pattern, segment);

        self.dirty = true;
        let id = Road::make(src, dst, segment, pattern, self);

        let inters = &mut self.intersections;

        inters[src].add_road(id, &self.roads);
        inters[dst].add_road(id, &self.roads);

        self.invalidate(src);
        self.invalidate(dst);

        Lot::remove_intersecting_lots(self, id);
        Lot::generate_along_road(self, id);
        Trees::remove_nearby_trees(self, id);

        id
    }

    pub fn build_buildings(&mut self) -> impl Iterator<Item = BuildingID> + '_ {
        info!("build buildings");
        self.dirty = true;

        let roads = &mut self.roads;
        let buildings = &mut self.buildings;
        let spatial_map = &mut self.spatial_map;

        self.lots.drain().filter_map(move |(id, lot)| {
            let rlots = &mut roads[lot.parent].lots;
            rlots.remove(rlots.iter().position(|&x| x == id).unwrap());
            spatial_map.remove(id);

            let r = rand::random::<f32>();

            let kind = if r < 0.1 {
                BuildingKind::Workplace
            } else if r < 0.2 {
                BuildingKind::Supermarket
            } else {
                BuildingKind::House
            };

            Building::make(buildings, spatial_map, roads, lot, kind)
        })
    }

    pub fn remove_road(&mut self, road_id: RoadID) -> Option<Road> {
        info!("remove_road {:?}", road_id);

        self.dirty = true;
        let road = self.roads.remove(road_id)?;

        self.spatial_map.remove(road_id);

        for (id, _) in road.lanes_iter() {
            self.lanes.remove(id);
            self.parking.remove_spots(id);
        }

        for &lot in &road.lots {
            self.lots.remove(lot);
            self.spatial_map.remove(lot);
        }

        self.intersections[road.src].remove_road(road_id);
        self.intersections[road.dst].remove_road(road_id);

        self.invalidate(road.src);
        self.invalidate(road.dst);
        Some(road)
    }

    pub fn clear(&mut self) {
        info!("clear");
        let before = std::mem::take(self);
        self.trees = before.trees;
    }

    pub fn project(&self, pos: Vec2) -> MapProject {
        let mk_proj = move |kind| MapProject { pos, kind };

        let mut qroad = None;
        for obj in self.spatial_map.query_point(pos) {
            match obj {
                ProjectKind::Inter(id) => {
                    let inter = self.intersections
                        .get(id)
                        .expect("Road does not exist anymore, you seem to have forgotten to remove it from the spatial map.");

                    if inter.polygon.contains(pos) {
                        return MapProject {
                            pos: inter.pos,
                            kind: obj,
                        };
                    }
                }
                ProjectKind::Lot(id) => {
                    if self.lots
                        .get(id)
                        .expect("Lot does not exist anymore, you seem to have forgotten to remove it from the spatial map.")
                        .shape
                        .contains(pos) {
                        return mk_proj(ProjectKind::Lot(id));
                    }
                }
                ProjectKind::Road(id) => {
                    if qroad.is_some() { continue; }
                    let road = self.roads
                        .get(id)
                        .expect("Road does not exist anymore, you seem to have forgotten to remove it from the spatial map.");

                    let projected = road.generated_points.project(pos);
                    if projected.is_close(pos, road.width * 0.5) {
                        qroad = Some((id, projected));
                    }
                },
                ProjectKind::Building(id) => {
                    if self.buildings
                        .get(id)
                        .expect("building does not exist anymore, you seem to have forgotten to remove it from the spatial map.")
                        .draw
                        .iter()
                        .any(|(p, _)| p.contains(pos)) {
                        return mk_proj(ProjectKind::Building(id));
                    }
                }
                _ => {},
            }
        }

        if let Some((id, pos)) = qroad {
            return MapProject {
                pos,
                kind: ProjectKind::Road(id),
            };
        }

        mk_proj(ProjectKind::Ground)
    }

    pub fn is_empty(&self) -> bool {
        self.roads.is_empty() && self.lanes.is_empty() && self.intersections.is_empty()
    }

    pub fn roads(&self) -> &Roads {
        &self.roads
    }
    pub fn lanes(&self) -> &Lanes {
        &self.lanes
    }
    pub fn intersections(&self) -> &Intersections {
        &self.intersections
    }
    pub fn buildings(&self) -> &Buildings {
        &self.buildings
    }
    pub fn lots(&self) -> &Lots {
        &self.lots
    }
    pub fn spatial_map(&self) -> &SpatialMap {
        &self.spatial_map
    }

    pub fn random_building<R: Rng>(&self, filter: BuildingKind, r: &mut R) -> Option<&Building> {
        self.buildings
            .iter()
            .filter(|(_, b)| b.kind == filter)
            .choose(r)
            .map(|x| x.1)
    }

    pub fn find_road(&self, src: IntersectionID, dst: IntersectionID) -> Option<RoadID> {
        for r in &self.intersections[src].roads {
            let road = &self.roads[*r];
            if road.src == src && road.dst == dst {
                return Some(road.id);
            }
        }
        None
    }

    pub fn nearest_lane(&self, p: Vec2, kind: LaneKind) -> Option<LaneID> {
        self.lanes
            .iter()
            .filter(|(_, x)| x.kind == kind)
            .min_by_key(|(_, lane)| OrderedFloat(lane.dist2_to(p)))
            .map(|(id, _)| id)
    }

    pub fn parking_to_drive(&self, spot: ParkingSpotID) -> Option<LaneID> {
        let spot = self.parking.get(spot)?;
        let park_lane = self
            .lanes
            .get(spot.parent)
            .expect("Parking spot has no parent >:(");
        let road = self
            .roads
            .get(park_lane.parent)
            .expect("Lane has no parent >:(");
        Some(
            road.outgoing_lanes_from(park_lane.src)
                .iter()
                .rfind(|&&(_, kind)| kind == LaneKind::Driving)
                .map(|&(id, _)| id)
                .expect("Road with parking lane doesn't have driving lane >:("),
        )
    }
}
