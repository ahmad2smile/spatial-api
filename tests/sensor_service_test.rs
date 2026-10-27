mod utils;

use std::collections::BTreeMap;

use spatial::sensor::{
    models::{SensorDto, SensorUpdateDto},
    service::SensorService,
};

fn sensor(name: &str, lat: f64, long: f64) -> SensorDto {
    SensorDto {
        name: name.into(),
        lat,
        long,
        tags: BTreeMap::from([("kind".into(), "temp".into())]),
    }
}

#[tokio::test]
async fn sensor_srv_tests() {
    let (_container, pool) = utils::create_db_pool().await;
    let srv = SensorService::new(pool);

    let sensors = [
        sensor("a", 40.0, 10.0),
        sensor("b", 10.0, 40.0),
        sensor("c", -30.0, -60.0),
    ];
    for s in &sensors {
        srv.create(s).await.unwrap();
    }

    let got = srv.get("c").await.unwrap();
    assert_eq!(got, sensors[2]);

    let update = SensorUpdateDto {
        lat: 10.0,
        long: 38.0,
        tags: BTreeMap::new(),
    };
    srv.update("c", &update).await.unwrap();

    let got = srv.get("c").await.unwrap();
    assert_eq!((got.lat, got.long), (10.0, 38.0));

    // c after update, should now be closer to this point than b
    let nearest = srv.nearby(10.0, 37.0).await.unwrap();
    assert_eq!(serde_json::to_value(nearest).unwrap()["name"], "c");
}
