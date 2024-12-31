#![allow(dead_code)] // unused constants

use ddcore_rs::memory::{GameConnection, ConnectionParams, OperatingSystem, MemoryOverride};
use ddcore_rs::models::{GameStatus, StatsDataBlock, replay::{DdRpl, EntityType, ReplayEvent::EnemyHitWeakSpot}};
use soloud::*;
#[allow(unused_imports)]
use std::fs::{File, OpenOptions};
#[allow(unused_imports)]
use std::io::{Cursor, Write};
use std::collections::BTreeMap;

const SKULL_I: usize = 0;
const SKULL_II: usize = 1;
const SKULL_III: usize = 2;
const SPIDERLING: usize = 3;
const SKULL_IV: usize = 4;
const SQUID_I: usize = 5;
const SQUID_II: usize = 6;
const SQUID_III: usize = 7;
const CENTIPEDE: usize = 8;
const GIGAPEDE: usize = 9;
const SPIDER_I: usize = 10;
const SPIDER_II: usize = 11;
const LEVIATHAN: usize = 12;
const ORB: usize = 13;
const THORN: usize = 14;
const GHOSTPEDE: usize = 15;
const SPIDER_EGG: usize = 16;

// is there really no way for the compiler to deduce the array size?
const WAVES: [u32; 38] = [0, 259, 350, 397, 455, 507, 556, 601, 642, 679, 714, 746, 776, 804, 830, 855, 878, 901, 922, 942, 962, 981, 999, 1016, 1032, 1048, 1064, 1079, 1093, 1108, 1121, 1134, 1147, 1160, 1172, 1184, 1195, 9999];

fn main() {
	let sl = Soloud::default().unwrap();
	let mut speech = audio::Speech::default();
	speech.set_text("gem lost").unwrap();
	let mut wav = audio::Wav::default();
	wav.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/metal-pipe-falling-sound-effect-By-Tuna.mp3"#))).unwrap();
	let mut wav2 = audio::Wav::default();
	wav2.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/burp_x.wav"#))).unwrap();
	let mut wav3 = audio::Wav::default();
	wav3.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/Microwave.mp3"#))).unwrap();
	let mut wav4 = audio::Wav::default();
	wav4.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/evaworciM.mp3"#))).unwrap();
	let mut use_homing = audio::Wav::default();
	//use_homing.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/homing.mp3"#))).unwrap();
	use_homing.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/boing.mp3"#))).unwrap();
	let mut boowomp = audio::Wav::default();
	boowomp.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/boowomp.mp3"#))).unwrap();
	let mut facepalm = audio::Wav::default();
	facepalm.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/facepalm.mp3"#))).unwrap();
	let mut shotguns = vec![];
	{
		let mut shotgun = audio::Wav::default();
		shotgun.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/shotgun1.wav"#))).unwrap();
		shotguns.push(shotgun);
	}
	{
		let mut shotgun = audio::Wav::default();
		shotgun.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/shotgun2.wav"#))).unwrap();
		shotguns.push(shotgun);
	}
	{
		let mut shotgun = audio::Wav::default();
		shotgun.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/shotgun3.wav"#))).unwrap();
		shotguns.push(shotgun);
	}
	{
		let mut shotgun = audio::Wav::default();
		shotgun.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/shotgun4.wav"#))).unwrap();
		shotguns.push(shotgun);
	}
	{
		let mut shotgun = audio::Wav::default();
		shotgun.load_mem(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), r#"/shotgun5.wav"#))).unwrap();
		shotguns.push(shotgun);
	}
	let mut logfile = None; //OpenOptions::new().create(true).write(true).append(true).open(r#"despawns.log"#).ok();
	loop {
		let mut connection = loop {
			let res = GameConnection::try_create(ConnectionParams {
				create_child: false,
				operating_system: OperatingSystem::Windows,
				overrides: MemoryOverride {
					process_name: Some("dd.exe".to_owned()),
					block_marker: Some(2452928),
				},
			});
			if let Ok(x) = res {
				break x;
			}
			//println!("Game not found... {:?}", res.err());
			std::thread::sleep(std::time::Duration::from_millis(1000));
		};
		std::thread::sleep(std::time::Duration::from_millis(3000));
		log(&mut logfile, "Connected to game");
		let mut last_data: Option<StatsDataBlock> = None;
		let mut last_shotgun_time = 0.;
		let mut shotgun_sum = 0;
		let mut shotgun_count = 0;
		let mut shotgun_average = 0.;
		let mut wave = 1;
		let mut wave_gems_lost = 0;
		loop {
			if !connection.is_alive() {
				break;
			}
			if let Ok(data_frames) = connection.read_stats_block_with_frames() {
				let data = data_frames.block;
				if let Some(last) = last_data {
					let curr_gems_lost = data.gems_despawned + data.gems_eaten;
					let last_gems_lost = last.gems_despawned + last.gems_eaten;
					if (data.status() == GameStatus::Playing || data.status() == GameStatus::OwnReplayFromLastRun || data.status() == GameStatus::OwnReplayFromLeaderboard || data.status() == GameStatus::OtherReplay || data.status() == GameStatus::LocalReplay) && last.status() != data.status() {
						log(&mut logfile, format!("Game started (at {:.0})", data.starting_time));
						wave = 1;
						wave_gems_lost = 0;
					}
					if (last.status() == GameStatus::Playing || last.status() == GameStatus::OwnReplayFromLastRun || last.status() == GameStatus::OwnReplayFromLeaderboard || last.status() == GameStatus::OtherReplay || last.status() == GameStatus::LocalReplay) && last.status() != data.status() {
						log(&mut logfile, format!("Game ended at {:.4} with {} gems lost and {} regushes. Shotgun avg {}", data.starting_time + last.time, last_gems_lost, regushes(&data), shotgun_average));
						let wave_start_time = WAVES[wave-1].max(data.starting_time as u32);
						log(&mut logfile, format!("Gems lost during {}-{} = {}", wave_start_time, (data.starting_time + last.time) as u32, wave_gems_lost));
						giga_info(&mut connection);
						//sl.play(&boowomp);
					}
					if data.status() == GameStatus::Playing || data.status() == GameStatus::OwnReplayFromLastRun || data.status() == GameStatus::OwnReplayFromLeaderboard || data.status() == GameStatus::OtherReplay || data.status() == GameStatus::LocalReplay {
						if last.status() == GameStatus::Playing && data.time < data.starting_time + 2. && last.time > last.starting_time + 5. {
							log(&mut logfile, format!("Game restarted at {:.4} with {} gems lost and {} regushes. Shotgun avg {} (new one starts at {:.0})", data.starting_time + last.time, last_gems_lost, regushes(&data), shotgun_average, data.starting_time));
							giga_info(&mut connection);
							wave = 1;
							wave_gems_lost = 0;
							//sl.play(&facepalm);
						}
						if curr_gems_lost > last_gems_lost {
							wave_gems_lost += curr_gems_lost - last_gems_lost;
							if data.time > last.time && data.time - last.time < 0.1 {
								log(&mut logfile, format!("Gem lost at {:.4}", data.starting_time + data.time));
								// if !(data.time > 350. && data.time < 405.) && !(data.starting_time < 300. && data.time > 660.) {
									sl.play(&wav);
								// }
							} else {
								log(&mut logfile, format!("Gem lost at {:.4} (skipping sound)", data.starting_time + data.time));
							}
						}
						if data.starting_time + data.time > 80. && data.starting_time + last.time <= 80. {
							// sl.play(&wav2);
							// play audio clip "start clearing arena"
							// etc do the rest of the important times too
						}
						for i in [716., 748., 778., 806., 832., 857., 880., 903., 924., 944., 964., 982., 1000., 1017., 1034., 1050., 1065., 1080., 1095., 1109., 1122., 1135., 1148., 1161.] {
							// https://discord.com/channels/399568958669455364/611994324300726281/1137206936685785180
							if data.starting_time + data.time > i && data.starting_time + last.time <= i {
								//sl.play(&use_homing);
							}
						}
						if last_gems_lost == curr_gems_lost && curr_gems_lost + data.gems_collected == gems_spawned(&data) && last_gems_lost + last.gems_collected != gems_spawned(&last) {
							//sl.play(&wav3);
						}
						if last_gems_lost == curr_gems_lost && curr_gems_lost + data.gems_collected != gems_spawned(&data) && last_gems_lost + last.gems_collected == gems_spawned(&last) {
							//sl.play(&wav4);
							// need something better here, like a long sound that can be stopped when they're collected
						}
						if last.daggers_fired + 10 <= data.daggers_fired {
							if data.time > last_shotgun_time {
								let frames_since_last = (((data.time - last_shotgun_time) * 60.).round() as usize).saturating_sub(20);
								if frames_since_last < shotguns.len() {
									//sl.play(&shotguns[frames_since_last]);
								}
								if frames_since_last < 20 {
									shotgun_sum += frames_since_last;
									shotgun_count += 1;
									shotgun_average = (shotgun_sum as f32) / (shotgun_count as f32);
								}
							}
							last_shotgun_time = data.time;
						}
						if data.starting_time + data.time >= WAVES[wave] as f32 {
							if data.starting_time + last.time < WAVES[wave] as f32 {
								let wave_start_time = WAVES[wave].max(data.starting_time as u32);
								log(&mut logfile, format!("Gems lost during {}-{} = {}", wave_start_time, WAVES[wave+1], wave_gems_lost));
							}
							wave_gems_lost = 0;
							while data.starting_time + data.time >= WAVES[wave] as f32 {
								wave += 1;
							}
						}
					}
				}
				last_data = Some(data);
			}
		}
		log(&mut logfile, "Disconnected from game");
		std::thread::sleep(std::time::Duration::from_millis(5000));
	}
}

fn gems_spawned(data: &StatsDataBlock) -> i32 {
	let gems_per_enemy_type: [i32; 17] = [0, 1, 1, 0, 1, 1, 2, 3, 25, 50, 1, 1, 6, 0, 0, 10, 0];
	let mut ret = 0;
	for i in 0..17 {
		ret += gems_per_enemy_type[i] * (data.per_enemy_kill_count[i] as i32);
	}
	ret
}

fn regushes(data: &StatsDataBlock) -> i16 {
	let sq1 = data.per_enemy_alive_count[SQUID_I]+data.per_enemy_kill_count[SQUID_I];
	let sq2 = data.per_enemy_alive_count[SQUID_II]+data.per_enemy_kill_count[SQUID_II];
	let sq3 = data.per_enemy_alive_count[SQUID_III]+data.per_enemy_kill_count[SQUID_III];
	let sk2 = data.per_enemy_alive_count[SKULL_II]+data.per_enemy_kill_count[SKULL_II];
	let sk3 = data.per_enemy_alive_count[SKULL_III]+data.per_enemy_kill_count[SKULL_III];
	let sk4 = data.per_enemy_alive_count[SKULL_IV]+data.per_enemy_kill_count[SKULL_IV];
	(sk2 + sk3 + sk4) - (sq1 + sq2 + sq3)
}

#[allow(unused_variables)]
fn log<S: Into<String>>(file: &mut Option<File>, x: S) {
	let x: String = x.into();
	println!("{}", x);
	//writeln!(file, "{}", x).unwrap();
}

fn giga_info(connection: &mut GameConnection) {
	let mut gigas: BTreeMap<i32, [i8; 50]> = BTreeMap::new();
	if let Ok(replay_bin) = connection.replay_bin() {
		if let Ok(mut replay) = DdRpl::from_reader(&mut Cursor::new(replay_bin)) {
			if let Ok(_) = replay.calc_data() {
				if let Some(data) = replay.data {
					gigas = data.entities.iter().filter_map(|p|
						match p.entity_type {
							EntityType::Gigapede => Some((p.id, [5; 50])),
							_ => None,
						}
					).collect();
					for frame in data.frames {
						for event in frame.events {
							match event {
								EnemyHitWeakSpot(hit) => {
									if let Some(giga) = gigas.get_mut(&hit.enemy_id) {
										giga[hit.segment as usize] -= 1;
									}
								}
								_ => (),
							}
						}
					}
				}
			}
		}
	}
	let mut any_alive = false;
	for g in gigas.values() {
		let mut alive = false;
		for &h in g {
			if h > 0 {
				alive = true;
				any_alive = true;
			}
		}
		if alive {
			for h in g {
				print!("{}", h);
			}
			println!();
		}
	}
	if any_alive {
		println!();
	}
}
