//! `PKT-044`: selection and hotkey squads against the reference dispatcher
//! (`tests/cpp_harness/gen_pkt044_dispatch.cpp`).
//!
//! `order_routing` is not a reference comparison: it states, from the C++ source,
//! which arguments and guards each routed order has.

use std::collections::BTreeMap;

use zh_common::golden_json::{extract_vector_lines, get_str, get_u32};
use zh_common::io::recorder::*;
use zh_common::runtime::game_logic_dispatch::*;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT-044.json");

#[derive(Clone, Copy)]
struct Obj {
    selectable: bool,
    mass: bool,
    joins: bool,
    owner: i32,
}

#[derive(Default)]
struct World {
    in_game: bool,
    objects: BTreeMap<u32, Obj>,
    players: [PlayerSquads; 2],
    calls: Vec<String>,
}

impl World {
    fn obj(&self, id: u32) -> Obj {
        self.objects[&id]
    }
}

impl CommandTarget for World {
    fn is_player(&self, player: i32) -> bool {
        (0..2).contains(&player)
    }
    fn is_in_game(&self) -> bool {
        self.in_game
    }
    fn object_exists(&self, id: u32) -> bool {
        self.objects.contains_key(&id)
    }
    fn is_selectable(&self, id: u32) -> bool {
        self.obj(id).selectable
    }
    fn is_mass_selectable(&self, id: u32) -> bool {
        self.obj(id).mass
    }
    fn can_join_group(&self, id: u32) -> bool {
        self.obj(id).joins
    }
    fn is_controlled_by(&self, id: u32, player: i32) -> bool {
        self.obj(id).owner == player
    }
    fn squads(&mut self, player: i32) -> &mut PlayerSquads {
        &mut self.players[player as usize]
    }
    fn set_rally_point(&mut self, _: &ReplayCommand, object: u32, pos: [f32; 3]) {
        self.calls.push(format!("rally {object} {pos:?}"));
    }
    fn do_special_power(&mut self, _: &ReplayCommand, group: &[u32], power: u32, options: u32) {
        self.calls
            .push(format!("power {group:?} {power} {options}"));
    }
    fn do_special_power_at_location(
        &mut self,
        _: &ReplayCommand,
        group: &[u32],
        power: u32,
        location: [f32; 3],
        angle: f32,
        object_in_way: Option<u32>,
        options: u32,
    ) {
        self.calls.push(format!(
            "power_at {group:?} {power} {location:?} {angle} {object_in_way:?} {options}"
        ));
    }
    fn do_special_power_at_object(
        &mut self,
        _: &ReplayCommand,
        group: &[u32],
        power: u32,
        target: u32,
        options: u32,
    ) {
        self.calls
            .push(format!("power_on {group:?} {power} {target} {options}"));
    }
    fn purchase_science(&mut self, _: &ReplayCommand, player: i32, science: i32) {
        self.calls.push(format!("science {player} {science}"));
    }
    fn queue_upgrade(&mut self, _: &ReplayCommand, group: &[u32], upgrade: u32) {
        self.calls.push(format!("upgrade {group:?} {upgrade}"));
    }
    fn cancel_upgrade(&mut self, _: &ReplayCommand, producer: u32, upgrade: u32) {
        self.calls
            .push(format!("cancel_upgrade {producer} {upgrade}"));
    }
    fn queue_unit_create(&mut self, _: &ReplayCommand, producer: u32, template: i32, id: i32) {
        self.calls.push(format!("build {producer} {template} {id}"));
    }
    fn cancel_unit_create(&mut self, _: &ReplayCommand, producer: u32, id: i32) {
        self.calls.push(format!("cancel_build {producer} {id}"));
    }
    fn sell(&mut self, _: &ReplayCommand, group: &[u32]) {
        self.calls.push(format!("sell {group:?}"));
    }
    fn logic_crc(&mut self, player: i32, crc: u32) {
        self.calls.push(format!("crc {player} {crc}"));
    }
    fn playback_logic_crc(&mut self, crc: u32) {
        self.calls.push(format!("playback_crc {crc}"));
    }
    fn unhandled(&mut self, msg: &ReplayCommand, group: Option<&[u32]>) {
        self.calls
            .push(format!("unhandled {} {group:?}", msg.msg_type));
    }
}

fn parse_message(text: &str) -> ReplayCommand {
    let mut f = text.split(' ');
    let mut number = || f.next().unwrap().parse::<i32>().unwrap(); // determinism-ok: reads the vector file
    let mut msg = ReplayCommand::new(number(), number());
    for arg in f {
        let (kind, value) = arg.split_once(':').unwrap();
        let value: i64 = value.parse().unwrap();
        msg.args.push(match kind {
            "0" => ReplayArg::integer(value as i32),
            "2" => ReplayArg::boolean(value != 0),
            "3" => ReplayArg::object_id(value as u32),
            other => panic!("kind {other}"),
        });
    }
    msg
}

fn dump(players: &[PlayerSquads; 2]) -> String {
    let join = |s: &Squad| {
        s.ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".")
    };
    let mut out = String::new();
    for p in players {
        out += &join(&p.selection);
        for h in &p.hotkeys {
            out.push('/');
            out += &join(h);
        }
        out.push('|');
    }
    out
}

/// The message numbers are those of the reference build (`PKT-444.json`).
#[test]
fn message_numbers() {
    let table = include_str!("../../../docs/golden_vectors/PKT-444.json");
    for (value, name) in [
        (MSG_CLEAR_GAME_DATA, "CLEAR_GAME_DATA"),
        (MSG_NEW_GAME, "NEW_GAME"),
        (MSG_LOGIC_CRC_PLAYBACK, "LOGIC_CRC_PLAYBACK"),
        (MSG_BEGIN_NETWORK_MESSAGES, "BEGIN_NETWORK_MESSAGES"),
        (MSG_CREATE_SELECTED_GROUP, "CREATE_SELECTED_GROUP"),
        (
            MSG_CREATE_SELECTED_GROUP_NO_SOUND,
            "CREATE_SELECTED_GROUP_NO_SOUND",
        ),
        (MSG_DESTROY_SELECTED_GROUP, "DESTROY_SELECTED_GROUP"),
        (MSG_REMOVE_FROM_SELECTED_GROUP, "REMOVE_FROM_SELECTED_GROUP"),
        (MSG_CREATE_TEAM0, "CREATE_TEAM0"),
        (MSG_SELECT_TEAM0, "SELECT_TEAM0"),
        (MSG_ADD_TEAM0, "ADD_TEAM0"),
        (MSG_DO_SPECIAL_POWER, "DO_SPECIAL_POWER"),
        (
            MSG_DO_SPECIAL_POWER_AT_LOCATION,
            "DO_SPECIAL_POWER_AT_LOCATION",
        ),
        (MSG_DO_SPECIAL_POWER_AT_OBJECT, "DO_SPECIAL_POWER_AT_OBJECT"),
        (MSG_SET_RALLY_POINT, "SET_RALLY_POINT"),
        (MSG_PURCHASE_SCIENCE, "PURCHASE_SCIENCE"),
        (MSG_QUEUE_UPGRADE, "QUEUE_UPGRADE"),
        (MSG_CANCEL_UPGRADE, "CANCEL_UPGRADE"),
        (MSG_QUEUE_UNIT_CREATE, "QUEUE_UNIT_CREATE"),
        (MSG_CANCEL_UNIT_CREATE, "CANCEL_UNIT_CREATE"),
        (MSG_SELL, "SELL"),
        (MSG_SET_REPLAY_CAMERA, "SET_REPLAY_CAMERA"),
        (MSG_LOGIC_CRC, "LOGIC_CRC"),
        (MSG_END_NETWORK_MESSAGES, "END_NETWORK_MESSAGES"),
    ] {
        let entry = format!("[{value}, \"MSG_{name}\"]");
        assert!(table.contains(&entry), "{entry}");
    }
}

#[test]
fn selection_and_hotkeys() {
    let rows = extract_vector_lines(VECTORS);
    assert_eq!(rows.len(), 900);
    let mut world = World::default();
    let (mut selected, mut hotkeyed) = (0, 0);
    for row in rows {
        let step = get_u32(row, "seed").unwrap();
        world.in_game = get_u32(row, "in_game") == Some(1);
        world.objects = get_str(row, "objects")
            .unwrap()
            .split_terminator(';')
            .map(|o| {
                let f: Vec<i64> = o.split(',').map(|v| v.parse().unwrap()).collect();
                let obj = Obj {
                    selectable: f[1] != 0,
                    mass: f[2] != 0,
                    joins: f[3] != 0,
                    owner: f[4] as i32,
                };
                (f[0] as u32, obj)
            })
            .collect();
        let msg = parse_message(get_str(row, "msg").unwrap());
        dispatch(&mut world, &msg);
        assert_eq!(
            dump(&world.players),
            get_str(row, "squads").unwrap(),
            "step {step}: {msg:?}"
        );
        selected += usize::from(!world.players[0].selection.ids.is_empty());
        hotkeyed += usize::from(world.players[1].hotkeys.iter().any(|h| !h.ids.is_empty()));
    }
    assert!(selected > 300 && hotkeyed > 300, "{selected} {hotkeyed}");
}

/// Objects 1 and 2 belong to player 0, object 3 to player 1; object 4 cannot join
/// a group.
fn orders_world() -> World {
    let obj = |owner, joins| Obj {
        selectable: true,
        mass: true,
        joins,
        owner,
    };
    let mut w = World {
        in_game: true,
        ..World::default()
    };
    w.objects = BTreeMap::from([
        (1, obj(0, true)),
        (2, obj(0, true)),
        (3, obj(1, true)),
        (4, obj(0, false)),
    ]);
    w
}

fn run(selection: &[u32], msg: ReplayCommand) -> Vec<String> {
    let mut w = orders_world();
    w.players[0].selection.ids = selection.to_vec();
    dispatch(&mut w, &msg);
    w.calls
}

fn order(msg_type: i32, args: &[ReplayArg]) -> ReplayCommand {
    ReplayCommand {
        msg_type,
        player_index: 0,
        args: args.to_vec(),
    }
}

#[test]
fn order_routing() {
    let int = ReplayArg::integer;
    let obj = ReplayArg::object_id;
    let place = ReplayArg::location(1.0, 2.0, 3.0);
    let sel: &[u32] = &[2, 3, 1];

    // Rally points name their object and need to own it.
    let rally = |o| order(MSG_SET_RALLY_POINT, &[obj(o), place]);
    assert_eq!(run(&[], rally(1)), ["rally 1 [1.0, 2.0, 3.0]"]);
    assert!(run(&[], rally(9)).is_empty());
    assert_eq!(run(&[], rally(3)).is_empty(), !cfg!(feature = "retail-crc"));

    // Group orders get the selection without foreign objects.
    assert_eq!(
        run(sel, order(MSG_QUEUE_UPGRADE, &[obj(0), int(77)])),
        ["upgrade [2, 1] 77"]
    );
    assert_eq!(run(sel, order(MSG_SELL, &[])), ["sell [2, 1]"]);
    assert!(run(&[3], order(MSG_SELL, &[])).is_empty());

    // Production uses the first selected object.
    assert_eq!(
        run(sel, order(MSG_QUEUE_UNIT_CREATE, &[int(5), int(6)])),
        ["build 2 5 6"]
    );
    assert_eq!(
        run(sel, order(MSG_CANCEL_UNIT_CREATE, &[int(8)])),
        ["cancel_build 2 8"]
    );
    assert_eq!(
        run(sel, order(MSG_CANCEL_UPGRADE, &[int(9)])),
        ["cancel_upgrade 2 9"]
    );
    assert!(run(&[], order(MSG_CANCEL_UPGRADE, &[int(9)])).is_empty());

    assert_eq!(
        run(&[], order(MSG_PURCHASE_SCIENCE, &[int(4)])),
        ["science 0 4"]
    );
    assert!(run(&[], order(MSG_PURCHASE_SCIENCE, &[int(-1)])).is_empty());

    // Special powers: a named source replaces the selection.
    let power = |source| order(MSG_DO_SPECIAL_POWER, &[int(10), int(20), obj(source)]);
    assert_eq!(run(sel, power(0)), ["power [2, 1] 10 20"]);
    assert!(run(&[], power(0)).is_empty());
    assert_eq!(run(sel, power(1)), ["power [1] 10 20"]);
    assert_eq!(run(sel, power(4)), ["power [] 10 20"]);
    assert_eq!(run(sel, power(3)).is_empty(), !cfg!(feature = "retail-crc"));

    let at_object = |target, source| {
        order(
            MSG_DO_SPECIAL_POWER_AT_OBJECT,
            &[int(11), obj(target), int(21), obj(source)],
        )
    };
    assert_eq!(run(sel, at_object(3, 1)), ["power_on [1] 11 3 21"]);
    assert_eq!(run(sel, at_object(3, 0)), ["power_on [2, 1] 11 3 21"]);
    assert!(run(sel, at_object(9, 1)).is_empty());

    // With six arguments the second is followed by an angle.
    let angle = ReplayArg::real(0.5);
    assert_eq!(
        run(
            sel,
            order(
                MSG_DO_SPECIAL_POWER_AT_LOCATION,
                &[int(12), place, angle, obj(3), int(22), obj(0)]
            )
        ),
        ["power_at [2, 1] 12 [1.0, 2.0, 3.0] 0.5 Some(3) 22"]
    );
    assert_eq!(
        run(
            sel,
            order(
                MSG_DO_SPECIAL_POWER_AT_LOCATION,
                &[int(12), place, obj(9), int(22), obj(1)]
            )
        ),
        ["power_at [1] 12 [1.0, 2.0, 3.0] -100 None 22"]
    );

    assert_eq!(run(&[], order(1095, &[int(-1)])), ["crc 0 4294967295"]);
    assert_eq!(
        run(&[], order(MSG_LOGIC_CRC_PLAYBACK, &[int(3)])),
        ["playback_crc 3"]
    );

    // Anything else is recorded, with the selection it would have used.
    assert_eq!(
        run(sel, order(1068, &[place])),
        ["unhandled 1068 Some([2, 1])"]
    );
    assert_eq!(run(sel, order(1092, &[place])), ["unhandled 1092 None"]);
    assert_eq!(run(sel, order(28, &[])), ["unhandled 28 None"]);
    let mut stranger = order(MSG_SELL, &[]);
    stranger.player_index = 5;
    assert_eq!(run(sel, stranger), ["unhandled 1052 None"]);

    for t in 0..2100 {
        let calls = run(sel, order(t, &[obj(1), obj(1), obj(1), obj(1), obj(1)]));
        if !is_routed(t) {
            assert_eq!(calls.len(), 1, "{t}");
            assert!(calls[0].starts_with(&format!("unhandled {t} ")), "{t}");
        } else {
            assert!(!calls.iter().any(|c| c.starts_with("unhandled")), "{t}");
        }
    }
}
