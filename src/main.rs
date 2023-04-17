#![allow(dead_code)] // unused constants

use ddcore_rs::memory::{GameConnection, ConnectionParams, OperatingSystem, MemoryOverride};
use ddcore_rs::models::{GameStatus, StatsDataBlock};
use soloud::*;
use std::fs::{File, OpenOptions};
use std::io::Write;

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

fn main() {
	let sl = Soloud::default().unwrap();
	let mut speech = audio::Speech::default();
	speech.set_text("gem lost").unwrap();
	let mut wav = audio::Wav::default();
	wav.load(&std::path::Path::new(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\metal-pipe-falling-sound-effect-By-Tuna.mp3"#)).unwrap();
	let mut wav2 = audio::Wav::default();
	wav2.load(&std::path::Path::new(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\burp_x.wav"#)).unwrap();
	let mut wav3 = audio::Wav::default();
	wav3.load(&std::path::Path::new(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\Microwave.mp3"#)).unwrap();
	let mut wav4 = audio::Wav::default();
	wav4.load(&std::path::Path::new(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\evaworciM.mp3"#)).unwrap();
	let mut shotguns = vec![];
	for i in 1..=5 {
		let mut shotgun = audio::Wav::default();
		shotgun.load(&std::path::Path::new(&format!(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\shotgun{}.wav"#, i))).unwrap();
		shotguns.push(shotgun);
	}
	let mut logfile = OpenOptions::new().create(true).write(true).append(true).open(r#"C:\Users\Ben\Desktop\code\ddgemdespawn\despawns.log"#).unwrap();
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
		let mut restart_eligible = false;
		let mut last_shotgun_time = 0.;
		let mut shotgun_sum = 0;
		let mut shotgun_count = 0;
		let mut shotgun_average = 0.;
		loop {
			if let Err(_) = connection.is_alive_res() {
				break;
			}
			if let Ok(data) = connection.read_stats_block() {
				if let Some(last) = last_data {
					let curr_gems_lost = data.gems_despawned + data.gems_eaten;
					let last_gems_lost = last.gems_despawned + last.gems_eaten;
					if restart_eligible && data.status() == GameStatus::Playing && last.status() == GameStatus::Playing && data.time < 2. && last.time > 5. {
						log(&mut logfile, format!("Game restarted at {:.4} with {} gems lost and {} regushes. Shotgun avg {} (new one starts at {:.0})", data.starting_time + last.time, last_gems_lost, regushes(&data), shotgun_average, data.starting_time));
					}
					restart_eligible = data.status() == GameStatus::Playing && data.time > 3.;
					if data.status() == GameStatus::Playing && last.status() != GameStatus::Playing {
						log(&mut logfile, format!("Game started (at {:.0})", data.starting_time));
					}
					if data.status() != GameStatus::Playing && last.status() == GameStatus::Playing {
						log(&mut logfile, format!("Game ended at {:.4} with {} gems lost and {} regushes. Shotgun avg {}", data.starting_time + last.time, last_gems_lost, regushes(&data), shotgun_average));
					}
					if data.status() == GameStatus::Playing && curr_gems_lost > last_gems_lost {
						log(&mut logfile, format!("Gem lost at {:.4}", data.starting_time + data.time));
						sl.play(&wav);
					}
					if data.starting_time + data.time > 80. && data.starting_time + last.time <= 80. {
						sl.play(&wav2);
						// play audio clip "start clearing arena"
						// etc do the rest of the important times too
					}
					if last_gems_lost == curr_gems_lost && curr_gems_lost + data.gems_collected == gems_spawned(&data) && last_gems_lost + last.gems_collected != gems_spawned(&last) {
						//sl.play(&wav3);
					}
					if last_gems_lost == curr_gems_lost && curr_gems_lost + data.gems_collected != gems_spawned(&data) && last_gems_lost + last.gems_collected == gems_spawned(&last) {
						//sl.play(&wav4);
						// need something better here, like a long sound that can be stopped when they're collected
					}
					if last.daggers_fired + 10 <= data.daggers_fired {
						let frames_since_last = ((data.time - last_shotgun_time) * 60.).round() as usize - 20;
						if frames_since_last < shotguns.len() {
							//sl.play(&shotguns[frames_since_last]);
						}
						if frames_since_last < 20 {
							shotgun_sum += frames_since_last;
							shotgun_count += 1;
							shotgun_average = (shotgun_sum as f32) / (shotgun_count as f32);
						}
						last_shotgun_time = data.time;
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

fn log<S: Into<String>>(file: &mut File, x: S) {
	let x: String = x.into();
	println!("{}", x);
	//writeln!(file, "{}", x).unwrap();
}
