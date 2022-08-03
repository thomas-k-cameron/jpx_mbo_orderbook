use crate::{
    CombinationProduct, DeleteOrder, EquilibriumPrice, Executed, ExecutionWithPriceInfo, LegPrice,
    ProductInfo, PutOrder, SecondTag, SystemEventInfo, TickSize, TradingStatusInfo,
};

#[test]
fn parse_put_order() {
    let list = [
        "A,2021-02-28T21:07:50.931282000(1614546470931282000),7395532366336496435,PUT_NK225_210312_19250(126484980),B,15,15,10000,0,2",
        "A,2021-02-28T21:07:50.931282000(1614546470931282000),7391176376074606176,PUT_NK225_210312_19250(126484980),B,6,3,10000,0,2",
        "A,2021-02-28T21:07:50.931282000(1614546470931282000),7389199179314731270,PUT_NK225_210611_17750(142017012),B,2,10,30000,0,2"
    ];
    for i in list {
        let item = PutOrder::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok())
    }
}

#[test]
fn parse_delete_order() {
    let list = [
        "D,2021-02-28T23:19:33.728095287(1614554373728095287),7396717914678617986,PUT_NK225_210312_29500(231080436),B",
        "D,2021-02-28T23:19:36.349389131(1614554376349389131),7396717914678616791,CAL_NK225_210312_31625(302580212),B",
        "D,2021-02-28T23:19:39.573310876(1614554379573310876),7396717914678617474,CAL_NK225_210312_29875(66585076),B",
        "D,2021-02-28T23:19:39.573792009(1614554379573792009),7396717914678614202,PUT_NK225_210312_27375(42336756),B",
        "D,2021-02-28T23:19:39.573992403(1614554379573992403),7396717914678616751,CAL_NK225_210305W_30250(275579380),B"
    ];
    for i in list {
        let item = DeleteOrder::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_combination_order() {
    let list = [
        "C,2021-03-01T00:09:42.006417851(1614557382006417851),7396717914679971014,PUT_NK225_210305W_29500(301531636),B,1,73967175152437406,0,,,3200000,Y,Y",
        "C,2021-03-01T00:09:42.006417851(1614557382006417851),7396717914679872948,PUT_NK225_210305W_29500(301531636),S,1,73967175152437406,0,,,3200000,Y,N",
        "C,2021-03-01T00:11:41.006070864(1614557501006070864),7396717914680253198,CAL_NK225_210305W_30125(273940980),B,1,73967175152437746,0,,,770000,Y,Y",
        "C,2021-03-01T00:11:41.006070864(1614557501006070864),7396717914680192740,CAL_NK225_210305W_30125(273940980),S,1,73967175152437746,0,,,770000,Y,N"
    ];
    for i in list {
        let item = ExecutionWithPriceInfo::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_executed() {
    let list = [
        "E,2021-03-01T00:06:20.042573706(1614557180042573706),7396717914678679169,CAL_NK225_210312_31000(283771380),B,1,73967175152436735,0,,",
        "E,2021-03-01T00:06:22.272433178(1614557182272433178),7396717914679503986,PUT_NK225_210312_26000(176685556),S,2,73967175152436736,0,,",
        "E,2021-03-01T00:06:22.407392231(1614557182407392231),7396717914678609278,PUT_NK225_210312_24500(156959220),B,1,73967175152436737,0,,",
        "E,2021-03-01T00:06:24.733744874(1614557184733744874),7396717914679518984,PUT_NK225_210312_27500(98238964),B,1,73967175152436740,0,,",
        "E,2021-03-01T00:06:25.003729506(1614557185003729506),7396717914679521996,PUT_NK225_210312_26000(176685556),B,1,73967175152436741,0,,",
        "E,2021-03-01T00:06:25.099005835(1614557185099005835),7396717914679522172,CAL_NK225_210312_30000(239337972),B,2,73967175152436742,0,,",
        "E,2021-03-01T00:06:25.329240470(1614557185329240470),7396717914679521996,PUT_NK225_210312_26000(176685556),B,1,73967175152436743,0,,",
        "E,2021-03-01T00:06:25.884069475(1614557185884069475),7396717914679516994,PUT_NK225_210312_29500(231080436),S,1,73967175152436744,0,,"
    ];

    for i in list {
        let item = Executed::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_tick_size() {
    let list = [
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),PUT_NK225_231208_14500(144835060),10000,10000,999999",
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),PUT_NK225_231208_14500(144835060),50000,1000000,9999999",
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),PUT_NK225_231208_14500(144835060),100000,10000000,999900000",
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),CAL_NK225_281208_26750(12124660),10000,10000,999999",
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),CAL_NK225_281208_26750(12124660),50000,1000000,9999999",
        "L,2021-02-28T21:07:50.931282000(1614546470931282000),CAL_NK225_281208_26750(12124660),100000,10000000,999900000"
    ];

    for i in list {
        let item = TickSize::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_combination_product() {
    // TODO
    unimplemented!()
}

#[test]
fn parse_trading_status_info() {
    let list = [
        "O,2021-02-28T23:00:01.019577797(1614553201019577797),PUT_NK225_210402W_30625(28246516),M_PRE_OPEN_NO_J-NET",
        "O,2021-02-28T23:00:01.019577797(1614553201019577797),PUT_NK225_210402W_30750(29491700),M_PRE_OPEN_NO_J-NET",
        "O,2021-02-28T23:00:01.019577797(1614553201019577797),PUT_NK225_210402W_30875(31392244),M_PRE_OPEN_NO_J-NET",
        "O,2021-02-28T23:00:01.019577797(1614553201019577797),PUT_NK225_210402W_31000(31523316),M_PRE_OPEN_NO_J-NET",
        "O,2021-02-28T23:00:01.019577797(1614553201019577797),PUT_NK225_210402W_31125(31588852),M_PRE_OPEN_NO_J-NET",
        "O,2021-02-28T23:20:01.050127428(1614554401050127428),CAL_NK225_210312_12500(45548020),M_PRE_OPEN",
        "O,2021-02-28T23:20:01.050127428(1614554401050127428),CAL_NK225_210312_12750(285868532),M_PRE_OPEN",
        "O,2021-02-28T23:20:01.050127428(1614554401050127428),CAL_NK225_210312_13000(278659572),M_PRE_OPEN",
        "O,2021-02-28T23:20:01.050127428(1614554401050127428),CAL_NK225_210312_13250(278725108),M_PRE_OPEN",
        "O,2021-02-28T23:20:01.050127428(1614554401050127428),CAL_NK225_210312_13500(263389684),M_PRE_OPEN",
    ];

    for i in list {
        let item = TradingStatusInfo::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_leg_price() {
    //todo
    unimplemented!()
}

#[test]
fn parse_product_info() {
    let list = [
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),40632820,PUT_NK225_210910_28000,186098018,186098018,1,JPY,4,0,0,1,0,0,0,500,28000,20210910,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),264700404,CAL_NK225_231208_14250,198124218,198124218,1,JPY,4,0,0,1,0,0,0,500,14250,20231208,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),223347188,CAL_NK225_210910_29500,196099518,196099518,1,JPY,4,0,0,1,0,0,0,500,29500,20210910,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),90636788,CAL_NK225_210910_21000,146091018,146091018,1,JPY,4,0,0,1,0,0,0,500,21000,20210910,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),314704372,CAL_NK225_210813_34500,196084518,196084518,1,JPY,4,0,0,1,0,0,0,500,34500,20210813,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),181993972,PUT_NK225_271210_19250,132129218,132129218,1,JPY,4,0,0,1,0,0,0,500,19250,20271210,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),49283572,PUT_NK225_210312_17500,136037518,136037518,1,JPY,4,0,0,1,0,0,0,500,17500,20210312,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),273351156,PUT_NK225_210305W_28375,136478320,136478320,1,JPY,4,0,0,1,0,0,0,500,28375,20210305,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),140640756,PUT_NK225_210611_12500,136062518,136062518,1,JPY,4,0,0,1,0,0,0,500,12500,20210611,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),7930356,CAL_NK225_210312_28375,146038318,146038318,1,JPY,4,0,0,1,0,0,0,500,28375,20210312,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),231997940,PUT_NK225_210813_23750,136083718,136083718,1,JPY,4,0,0,1,0,0,0,500,23750,20210813,0,2",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),99287540,CAL_NK225_250613_32000,190062018,190062018,1,JPY,4,0,0,1,0,0,0,500,32000,20250613,0,1",
        "R,2021-02-28T21:07:50.931282000(1614546470931282000),323355124,PUT_NK225_210409_35250,186045218,186045218,1,JPY,4,0,0,1,0,0,0,500,35250,20210409,0,2"
    ];

    for i in list {
        let item = ProductInfo::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_system_event_info() {
    let list = [
        "S,2021-02-28T21:07:50.931282000(1614546470931282000),O",
        "S,2021-03-01T20:47:47.459260033(1614631667459260033),C",
    ];

    for i in list {
        let item = SystemEventInfo::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_time() {
    let list = [
        "T,1614557640",
        "T,1614557641",
        "T,1614557642",
        "T,1614557643",
        "T,1614557644",
        "T,1614557645",
        "T,1614557646",
        "T,1614557647",
    ];

    for i in list {
        let item = SecondTag::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}

#[test]
fn parse_equilibrium_price() {
    let list = [
        "Z,2021-03-01T00:01:46.776038698(1614556906776038698),PUT_NK225_210312_29375(188809716),0,0,-2147483648,3850000,3950000,3,2",
        "Z,2021-03-01T00:01:46.885442765(1614556906885442765),PUT_NK225_210312_29375(188809716),0,0,-2147483648,3850000,3950000,3,8",
        "Z,2021-03-01T00:01:46.885795538(1614556906885795538),PUT_NK225_210312_29375(188809716),0,0,-2147483648,3850000,3950000,3,2",
        "Z,2021-03-01T00:01:48.872876750(1614556908872876750),PUT_NK225_210312_29375(188809716),0,0,-2147483648,3850000,3950000,3,7",
        "Z,2021-03-01T00:01:49.152432026(1614556909152432026),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3100000,3150000,6,2",
        "Z,2021-03-01T00:01:50.123475866(1614556910123475866),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3050000,3150000,30,2",
        "Z,2021-03-01T00:01:50.124456116(1614556910124456116),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3050000,3150000,30,8",
        "Z,2021-03-01T00:01:50.125140358(1614556910125140358),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3050000,3150000,30,2",
        "Z,2021-03-01T00:01:50.379536464(1614556910379536464),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3050000,3150000,30,8",
        "Z,2021-03-01T00:01:50.405271228(1614556910405271228),PUT_NK225_210312_29125(188744180),0,0,-2147483648,3050000,3150000,30,2",
        "Z,2021-03-01T00:01:51.491574338(1614556911491574338),PUT_NK225_210312_26125(41812468),0,0,-2147483648,340000,350000,1,2",
        "Z,2021-03-01T00:01:51.491991229(1614556911491991229),PUT_NK225_210312_26125(41812468),1,1,340000,340000,340000,1,1",
        "Z,2021-03-01T00:01:51.492010505(1614556911492010505),PUT_NK225_210312_26125(41812468),1,2,340000,340000,340000,1,2",
        "Z,2021-03-01T00:01:51.492019390(1614556911492019390),PUT_NK225_210312_26125(41812468),1,3,340000,340000,340000,1,3"
    ];

    for i in list {
        let item = EquilibriumPrice::try_from(i);
        println!("{:?}", item);
        assert!(item.is_ok());
    }
}
