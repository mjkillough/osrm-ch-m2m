use bitflags::bitflags;

use super::storage::RectangleInt2D;
use crate::coordinates::{squared_euclidian_distance, Coordinate, FixedLatitude, FixedLongitude};

impl RectangleInt2D {
    // Assumes we're operating in euclidian space.
    pub fn get_min_squared_dist(&self, coordinate: &Coordinate) -> u64 {
        if self.contains(coordinate) {
            return 0;
        }

        bitflags! {
            struct Direction: u8 {
                const INVALID = 0;
                const NORTH = 1;
                const SOUTH = 2;
                const EAST = 4;
                const NORTH_EAST = 5;
                const SOUTH_EAST = 6;
                const WEST = 8;
                const NORTH_WEST = 9;
                const SOUTH_WEST = 10;
            }
        }

        let mut direction = Direction::INVALID;
        if coordinate.latitude > self.max_latitude() {
            direction = direction | Direction::NORTH;
        } else if coordinate.latitude < self.min_latitude() {
            direction |= Direction::SOUTH;
        }
        if coordinate.longitude > self.max_longitude() {
            direction |= Direction::EAST;
        } else if coordinate.longitude < self.min_longitude() {
            direction |= Direction::WEST;
        }

        match direction {
            Direction::NORTH => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: coordinate.longitude,
                    latitude: self.max_latitude(),
                },
            ),
            Direction::SOUTH => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: coordinate.longitude,
                    latitude: self.min_latitude(),
                },
            ),
            Direction::WEST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.min_longitude(),
                    latitude: coordinate.latitude,
                },
            ),
            Direction::EAST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.max_longitude(),
                    latitude: coordinate.latitude,
                },
            ),
            Direction::NORTH_EAST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.max_longitude(),
                    latitude: self.max_latitude(),
                },
            ),
            Direction::NORTH_WEST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.min_longitude(),
                    latitude: self.max_latitude(),
                },
            ),
            Direction::SOUTH_EAST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.max_longitude(),
                    latitude: self.min_latitude(),
                },
            ),
            Direction::SOUTH_WEST => squared_euclidian_distance(
                coordinate,
                &Coordinate {
                    longitude: self.min_longitude(),
                    latitude: self.min_latitude(),
                },
            ),
            _ => panic!("invalid direction"),
        }
    }

    fn contains(&self, coordinate: &Coordinate) -> bool {
        let lons_contained = (coordinate.longitude >= self.min_longitude())
            && (coordinate.longitude <= self.max_longitude());
        let lats_contained = (coordinate.latitude >= self.min_latitude())
            && (coordinate.latitude <= self.max_latitude());
        lons_contained && lats_contained
    }

    fn min_longitude(&self) -> FixedLongitude {
        FixedLongitude(self.min_lon)
    }

    fn max_longitude(&self) -> FixedLongitude {
        FixedLongitude(self.max_lon)
    }

    fn min_latitude(&self) -> FixedLatitude {
        FixedLatitude(self.min_lon)
    }

    fn max_latitude(&self) -> FixedLatitude {
        FixedLatitude(self.max_lon)
    }
}
