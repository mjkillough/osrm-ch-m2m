use super::{FloatCoordinate, FloatLatitude, FloatLongitude};

const DEGREE_TO_RAD: f64 = 0.017453292519943295769236907684886;
const RAD_TO_DEGREE: f64 = 1. / DEGREE_TO_RAD;
const EPSG3857_MAX_LATITUDE: f64 = 85.051128779806592378;

fn clamp(latitude: FloatLatitude) -> FloatLatitude {
    FloatLatitude(f64::max(
        f64::min(latitude.0, EPSG3857_MAX_LATITUDE),
        -EPSG3857_MAX_LATITUDE,
    ))
}

fn latitude_to_y(latitude: FloatLatitude) -> FloatLatitude {
    let clamped_latitude = clamp(latitude);
    let f = f64::sin(DEGREE_TO_RAD * clamped_latitude.0);
    FloatLatitude(RAD_TO_DEGREE * 0.5 * f64::ln((1. + f) / (1. - f)))
}

fn horner(x: f64, a: &[f64]) -> f64 {
    a.iter().fold(0., |acc, an| acc * x + an)
}

pub fn latitude_to_y_approx(latitude: FloatLatitude) -> FloatLatitude {
    if latitude < FloatLatitude(-70.) || latitude > FloatLatitude(70.) {
        return latitude_to_y(latitude);
    }

    FloatLatitude(
        horner(
            latitude.0,
            &[
                0.00000000000000000000000000e+00,
                1.00000000000089108431373566e+00,
                2.34439410386997223035693483e-06,
                -3.21291701673364717170998957e-04,
                -6.62778508496089940141103135e-10,
                3.68188055470304769936079078e-08,
                6.31192702320492485752941578e-14,
                -1.77274453235716299127325443e-12,
                -2.24563810831776747318521450e-18,
                3.13524754818073129982475171e-17,
                2.09014225025314211415458228e-23,
                -9.82938075991732185095509716e-23,
            ],
        ) / horner(
            latitude.0,
            &[
                1.00000000000000000000000000e+00,
                2.34439410398970701719081061e-06,
                -3.72061271627251952928813333e-04,
                -7.81802389685429267252612620e-10,
                5.18418724186576447072888605e-08,
                9.37468561198098681003717477e-14,
                -3.30833288607921773936702558e-12,
                -4.78446279888774903983338274e-18,
                9.32999229169156878168234191e-17,
                9.17695141954265959600965170e-23,
                -8.72130728982012387640166055e-22,
                -3.23083224835967391884404730e-28,
            ],
        ),
    )
}
