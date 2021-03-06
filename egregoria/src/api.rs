use crate::economy::{Market, Transaction};
use crate::map_dynamic::{BuildingInfos, Itinerary, ParkingManagement};
use crate::pedestrians::data::PedestrianID;
use crate::pedestrians::put_pedestrian_in_coworld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::{put_vehicle_in_coworld, Vehicle, VehicleID, VehicleState};
use crate::{Egregoria, ParCommandBuffer, SoulID};
use geom::{Spline, Transform, Vec2};
use imgui_inspect_derive::*;
use legion::Entity;
use map_model::{BuildingID, CarPath, Map, ParkingSpotID, PedestrianPath};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}

debug_inspect_impl!(Location);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    Outside(Vec2),
    Building(BuildingID),
}

debug_inspect_impl!(Destination);

#[derive(Copy, Clone, Debug)]
pub enum RoutingStep {
    WalkTo(Vec2),
    DriveTo(VehicleID, Vec2),
    Park(VehicleID, ParkingSpotID),
    Unpark(VehicleID),
    GetInVehicle(VehicleID),
    GetOutVehicle(VehicleID),
    GetInBuilding(BuildingID),
    GetOutBuilding(BuildingID),
}

debug_inspect_impl!(RoutingStep);

impl RoutingStep {
    pub fn ready(&self, goria: &Egregoria, body: PedestrianID) -> bool {
        let pos = goria.pos(body.0).unwrap();
        match self {
            RoutingStep::WalkTo(_) => true,
            RoutingStep::DriveTo(_, _) => true,
            RoutingStep::Park(vehicle, _) => {
                goria.comp::<Itinerary>(vehicle.0).unwrap().has_ended(0.0)
            }
            RoutingStep::Unpark(_) => true,
            RoutingStep::GetInVehicle(vehicle) => goria.pos(vehicle.0).unwrap().is_close(pos, 3.0),
            RoutingStep::GetOutVehicle(vehicle) => matches!(
                goria.comp::<Vehicle>(vehicle.0).unwrap().state,
                VehicleState::Parked(_)
            ),
            &RoutingStep::GetInBuilding(build) => goria.read::<Map>().buildings()[build]
                .door_pos
                .is_close(pos, 3.0),
            RoutingStep::GetOutBuilding(_) => true,
        }
    }
    pub fn action(self, goria: &Egregoria, body: PedestrianID) -> Option<Action> {
        Some(match self {
            RoutingStep::WalkTo(obj) => {
                let pos = goria.pos(body.0).unwrap();

                let map = goria.read::<Map>();

                if let Some(itin) = Itinerary::route(pos, obj, &*map, &PedestrianPath) {
                    Action::Navigate(body.0, itin)
                } else {
                    Action::DoNothing
                }
            }
            RoutingStep::DriveTo(vehicle, obj) => {
                let pos = goria.pos(vehicle.0).unwrap();

                let map = goria.read::<Map>();

                if let Some(itin) = Itinerary::route(pos, obj, &*map, &CarPath) {
                    Action::Navigate(vehicle.0, itin)
                } else {
                    Action::DoNothing
                }
            }
            RoutingStep::Park(vehicle, spot) => {
                if !goria.read::<Map>().parking.contains(spot) {
                    return None;
                }
                Action::Park(vehicle, spot)
            }
            RoutingStep::Unpark(vehicle) => Action::UnPark(vehicle),
            RoutingStep::GetInVehicle(vehicle) => Action::GetInVehicle(body, vehicle),
            RoutingStep::GetOutVehicle(vehicle) => Action::GetOutVehicle(body, vehicle),
            RoutingStep::GetInBuilding(build) => Action::GetInBuilding(body, build),
            RoutingStep::GetOutBuilding(build) => Action::GetOutBuilding(body, build),
        })
    }
}

#[derive(Clone, Inspect)]
pub struct Router {
    pub body: PedestrianID,
    car: Option<VehicleID>,
    steps: Vec<RoutingStep>,
    dest: Option<Destination>,
}

impl Router {
    pub fn new(body: PedestrianID, car: Option<VehicleID>) -> Self {
        Self {
            body,
            steps: vec![],
            car,
            dest: None,
        }
    }

    pub fn arrived(&self, destination: Destination) -> bool {
        if let Some(dest) = self.dest {
            dest == destination && self.steps.is_empty()
        } else {
            false
        }
    }

    pub fn body_pos(&self, goria: &Egregoria) -> Vec2 {
        goria.pos(self.body.0).unwrap()
    }

    fn clear_steps(&mut self, goria: &Egregoria) {
        for s in self.steps.drain(..) {
            if let RoutingStep::Park(_, spot) = s {
                goria.read::<ParkingManagement>().free(spot);
            }
        }
    }

    pub fn go_to(&mut self, goria: &Egregoria, dest: Destination) -> Action {
        if self.dest.map(|x| x == dest).unwrap_or(false) {
            if let Some(action) = self.action(goria) {
                return action;
            }
        }

        self.dest = Some(dest);

        self.clear_steps(goria);
        match dest {
            Destination::Outside(pos) => self.steps = self.steps_to(goria, pos),
            Destination::Building(build) => {
                let loc = goria.comp::<Location>(self.body.0).unwrap();
                if let Location::Building(cur_build) = loc {
                    if *cur_build == build {
                        return Action::DoNothing;
                    }
                }

                let door_pos = goria.read::<Map>().buildings()[build].door_pos;
                self.steps = self.steps_to(goria, door_pos);
                self.steps.push(RoutingStep::GetInBuilding(build));
            }
        }

        self.steps.reverse();

        self.action(goria).unwrap_or(Action::DoNothing)
    }

    fn steps_to(&self, goria: &Egregoria, obj: Vec2) -> Vec<RoutingStep> {
        let mut steps = vec![];
        let loc = goria.comp::<Location>(self.body.0).unwrap();
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }

        if let Some(car) = self.car {
            let map = goria.read::<Map>();
            if let Some(spot_id) = goria.read::<ParkingManagement>().reserve_near(obj, &map) {
                let lane = map.parking_to_drive(spot_id).unwrap();
                let spot = *goria.read::<Map>().parking.get(spot_id).unwrap();

                let (pos, _, dir) = map.lanes()[lane]
                    .points
                    .project_segment_dir(spot.trans.position());
                let parking_pos = pos - dir * 4.0;

                if !matches!(loc, Location::Vehicle(_)) {
                    steps.push(RoutingStep::WalkTo(goria.pos(car.0).unwrap()));
                    steps.push(RoutingStep::GetInVehicle(car));
                    steps.push(RoutingStep::Unpark(car));
                }

                steps.push(RoutingStep::DriveTo(car, parking_pos));
                steps.push(RoutingStep::Park(car, spot_id));
                steps.push(RoutingStep::GetOutVehicle(car));
            }
        }

        steps.push(RoutingStep::WalkTo(obj));
        steps
    }

    pub fn action(&mut self, goria: &Egregoria) -> Option<Action> {
        let step = unwrap_or!(self.steps.last(), return Some(Action::DoNothing));
        if step.ready(goria, self.body) {
            let step = self.steps.pop().unwrap();
            return step.action(goria, self.body);
        }
        Some(Action::DoNothing)
    }
}

#[derive(Debug)]
pub enum Action {
    DoNothing,
    GetOutBuilding(PedestrianID, BuildingID),
    GetInBuilding(PedestrianID, BuildingID),
    GetOutVehicle(PedestrianID, VehicleID),
    GetInVehicle(PedestrianID, VehicleID),
    Navigate(Entity, Itinerary),
    Park(VehicleID, ParkingSpotID),
    UnPark(VehicleID),
    Buy {
        buyer: SoulID,
        seller: SoulID,
        trans: Transaction,
    },
}

impl Default for Action {
    fn default() -> Self {
        Self::DoNothing
    }
}

impl Action {
    pub fn apply(self, goria: &mut Egregoria) -> Option<()> {
        match self {
            Action::DoNothing => {}
            Action::GetOutBuilding(body, building) => {
                log::info!("{:?}", self);
                goria.write::<BuildingInfos>().get_out(building, body);
                let wpos = goria.read::<Map>().buildings()[building].door_pos;
                walk_outside(goria, body, wpos);
            }
            Action::GetInBuilding(body, building) => {
                log::info!("{:?}", self);
                goria.write::<BuildingInfos>().get_in(building, body);
                *goria.comp_mut::<Location>(body.0).unwrap() = Location::Building(building);
                walk_inside(goria, body);
            }
            Action::GetOutVehicle(body, vehicle) => {
                log::info!("{:?}", self);
                let trans = *goria.comp::<Transform>(vehicle.0).unwrap();
                walk_outside(
                    goria,
                    body,
                    trans.position() + trans.direction().perpendicular() * 2.0,
                );
            }
            Action::GetInVehicle(body, vehicle) => {
                log::info!("{:?}", self);
                *goria.comp_mut::<Location>(body.0).unwrap() = Location::Vehicle(vehicle);
                walk_inside(goria, body);
            }
            Action::Navigate(e, itin) => {
                log::info!("Navigate {:?}", e);
                if let Some(v) = goria.comp_mut(e) {
                    *v = itin;
                } else {
                    log::warn!("Called navigate on entity that doesn't have itinerary component");
                }
            }
            Action::Park(vehicle, spot_id) => {
                log::info!("{:?}", self);
                let trans = goria.comp::<Transform>(vehicle.0).unwrap();
                let map = goria.read::<Map>();
                let spot = match map.parking.get(spot_id) {
                    Some(x) => x,
                    None => {
                        log::warn!("Couldn't park at {:?} because it doesn't exist", spot_id);
                        return None;
                    }
                };

                let s = Spline {
                    from: trans.position(),
                    to: spot.trans.position(),
                    from_derivative: trans.direction() * 2.0,
                    to_derivative: spot.trans.direction() * 2.0,
                };
                drop(map);

                goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state =
                    VehicleState::RoadToPark(s, 0.0, spot_id);
                goria.comp_mut::<Kinematics>(vehicle.0).unwrap().velocity = Vec2::ZERO;
            }
            Action::UnPark(vehicle) => {
                log::info!("{:?}", self);
                let v = goria.comp::<Vehicle>(vehicle.0).unwrap();
                let w = v.kind.width();

                if let VehicleState::Parked(spot) = v.state {
                    goria.read::<ParkingManagement>().free(spot);
                } else {
                    log::warn!("Trying to unpark {:?} that wasn't parked", vehicle);
                }

                let coll =
                    put_vehicle_in_coworld(goria, w, *goria.comp::<Transform>(vehicle.0).unwrap());
                goria
                    .read::<ParCommandBuffer>()
                    .add_component(vehicle.0, coll);

                goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state = VehicleState::Driving;
            }
            Action::Buy {
                buyer,
                seller,
                trans,
            } => {
                log::info!("{:?}", self);
                goria.write::<Market>().apply(buyer, seller, trans);
            }
        }
        Some(())
    }
}

fn walk_inside(goria: &mut Egregoria, body: PedestrianID) {
    let body = body.0;
    goria.comp_mut::<MeshRender>(body).unwrap().hide = true;
    goria
        .read::<ParCommandBuffer>()
        .remove_component::<Collider>(body);
    goria.comp_mut::<Kinematics>(body).unwrap().velocity = Vec2::ZERO;
    *goria.comp_mut::<Itinerary>(body).unwrap() = Itinerary::none();
}

fn walk_outside(goria: &mut Egregoria, body: PedestrianID, pos: Vec2) {
    let body = body.0;
    *goria.comp_mut::<Location>(body).unwrap() = Location::Outside;
    goria.comp_mut::<Transform>(body).unwrap().set_position(pos);
    goria.comp_mut::<MeshRender>(body).unwrap().hide = false;
    let coll = put_pedestrian_in_coworld(goria, pos);
    goria.read::<ParCommandBuffer>().add_component(body, coll);
}
