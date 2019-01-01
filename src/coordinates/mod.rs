mod web_mercator;

// include/util/coordinate.hpp

const COORDINATE_PRECISION: f64 = 1e6;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixedLongitude(pub i32);
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixedLatitude(pub i32);
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct FloatLongitude(pub f64);
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct FloatLatitude(pub f64);

macro_rules! impl_from_float {
    ($fixed:ident, $float:ident) => {
        impl From<$float> for $fixed {
            fn from(other: $float) -> $fixed {
                $fixed((other.0 * COORDINATE_PRECISION).round() as i32)
            }
        }
    };
}

impl_from_float!(FixedLongitude, FloatLongitude);
impl_from_float!(FixedLatitude, FloatLatitude);

macro_rules! impl_from_fixed {
    ($float:ident, $fixed:ident) => {
        impl From<$fixed> for $float {
            fn from(other: $fixed) -> $float {
                $float(f64::from(other.0) / COORDINATE_PRECISION)
            }
        }
    };
}

impl_from_fixed!(FloatLongitude, FixedLongitude);
impl_from_fixed!(FloatLatitude, FixedLatitude);

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Coordinate {
    pub longitude: FixedLongitude,
    pub latitude: FixedLatitude,
}

impl From<FloatCoordinate> for Coordinate {
    fn from(other: FloatCoordinate) -> Coordinate {
        Coordinate {
            longitude: other.longitude.into(),
            latitude: other.latitude.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct FloatCoordinate {
    pub longitude: FloatLongitude,
    pub latitude: FloatLatitude,
}

impl From<Coordinate> for FloatCoordinate {
    fn from(other: Coordinate) -> FloatCoordinate {
        FloatCoordinate {
            longitude: other.longitude.into(),
            latitude: other.latitude.into(),
        }
    }
}

impl FloatCoordinate {
    pub fn from_wgs84(&self) -> FloatCoordinate {
        FloatCoordinate {
            longitude: self.longitude,
            latitude: web_mercator::latitude_to_y_approx(self.latitude),
        }
    }
}

pub fn project_point_on_segment(
    source: &FloatCoordinate,
    target: &FloatCoordinate,
    coordinate: &FloatCoordinate,
) -> FloatCoordinate {
    let slope_vector = FloatCoordinate {
        longitude: FloatLongitude(target.longitude.0 - source.longitude.0),
        latitude: FloatLatitude(target.latitude.0 - source.latitude.0),
    };
    let rel_coordinate = FloatCoordinate {
        longitude: FloatLongitude(coordinate.longitude.0 - source.longitude.0),
        latitude: FloatLatitude(coordinate.latitude.0 - source.latitude.0),
    };

    // Dot product of two un-normed vectors:
    let unnormed_ratio = slope_vector.longitude.0 * rel_coordinate.longitude.0
        + slope_vector.latitude.0 * rel_coordinate.latitude.0;

    // Squared Length of the slope vector:
    let squared_length = slope_vector.longitude.0 * slope_vector.longitude.0
        + slope_vector.latitude.0 * slope_vector.latitude.0;

    if squared_length < std::f64::EPSILON {
        return *source;
    }

    let normed_ratio = unnormed_ratio / squared_length;
    let clamped_ratio = if unnormed_ratio > 1. {
        1.
    } else if unnormed_ratio < 0. {
        0.
    } else {
        normed_ratio
    };

    let result_longitude =
        (1. - clamped_ratio) * source.longitude.0 + target.longitude.0 * clamped_ratio;
    let result_latitude =
        (1. - clamped_ratio) * source.latitude.0 + target.latitude.0 * clamped_ratio;

    FloatCoordinate {
        longitude: FloatLongitude(result_longitude),
        latitude: FloatLatitude(result_latitude),
    }
}

// Assumes we're operating in euclidian space.
pub fn squared_euclidian_distance(lhs: &Coordinate, rhs: &Coordinate) -> u64 {
    let delta_longitude: i64 = i64::from(lhs.longitude.0) - i64::from(rhs.longitude.0);
    let delta_latitude: i64 = i64::from(lhs.latitude.0) - i64::from(rhs.latitude.0);

    let squared_longitude = delta_longitude * delta_longitude;
    let squared_latitude = delta_latitude * delta_latitude;

    (squared_latitude + squared_longitude) as u64
}
