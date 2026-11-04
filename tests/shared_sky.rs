// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "sky-timeline")]

use cleromancy::sky::daily::{jiff, sky_receipt::SkyReceiptV1, sky_timeline::*, turquet};
use turquet::apparent::ApparentBody;
use turquet::events::{ConventionalRiseSetPolicy, HorizonDipModel, LimbModel, RefractionModel};
use turquet::foundation::{
    EastLongitude, JulianDate, Latitude, Length, Observer, ScaleAwareEpoch, TimeOffset,
};
use turquet::observer::EarthOrientation;
use turquet::provider::ConstantOffsetEarthOrientation;

#[test]
fn cleromancy_uses_the_shared_named_zone_timeline_and_current_receipt_identity() {
    let request = SkyDayRequest::new(
        jiff::civil::date(2024, 4, 8),
        "America/New_York",
        Observer::new(
            EastLongitude::from_degrees(-71.0589).unwrap(),
            Latitude::from_degrees(42.3601).unwrap(),
            Length::from_meters(0.0).unwrap(),
        ),
        jiff::civil::time(12, 0, 0, 0),
        [ApparentBody::Sun, ApparentBody::Moon]
            .into_iter()
            .map(|body| {
                SkyBodySelection::new(
                    body,
                    ConventionalRiseSetPolicy::new(
                        RefractionModel::none(),
                        LimbModel::center(),
                        HorizonDipModel::level(),
                    ),
                )
            })
            .collect(),
        SkySearchControls::one_hour_one_second(),
    )
    .unwrap();
    let day = request.resolve_day().unwrap();
    // New York midnight was 04:00 UTC. The approximation is explicit.
    let epoch = ScaleAwareEpoch::from_gregorian_utc(2024, 4, 8, 4, 0, 0, 0);
    let ut1 = JulianDate::from_utc_epoch(epoch, TimeOffset::from_seconds(0.0).unwrap());
    let eop = ConstantOffsetEarthOrientation::new(
        day.start_tt(),
        EarthOrientation::zero_polar_motion(ut1, "shared sky test", "UT1=UTC; xp=yp=0"),
    );
    let timeline = cleromancy::analytical_local_sky_day(&request, &eop).unwrap();
    assert_eq!(
        timeline.receipt().day().time_zone_name(),
        "America/New_York"
    );
    assert!(
        timeline
            .items()
            .iter()
            .any(|item| matches!(item, SkyTimelineItem::LunarPhase(_)))
    );
    assert!(
        timeline
            .items()
            .iter()
            .any(|item| matches!(item, SkyTimelineItem::RiseSet(_)))
    );
    assert!(
        timeline
            .items()
            .iter()
            .any(|item| matches!(item, SkyTimelineItem::MeridianTransit(_)))
    );
    let receipt = SkyReceiptV1::from_timeline(&timeline);
    assert_eq!(receipt.engine.version, "0.17.0");
    assert_eq!(
        SkyReceiptV1::from_json(&receipt.to_pretty_json().unwrap()).unwrap(),
        receipt
    );
}
