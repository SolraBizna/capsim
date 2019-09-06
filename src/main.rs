#[macro_use]
extern crate lazy_static;

extern crate getopts;
extern crate num_cpus;
extern crate rand;

use rand::{Rng,SeedableRng};
use std::sync::Arc;
use std::time::{Duration,Instant};

mod power_level;
use power_level::*;
use PowerLevel as Weapon;
use PowerLevel as Government;

mod capture_odds;
use capture_odds::*;

mod invocation;
use invocation::*;

mod doomsday_clock;
use doomsday_clock::*;

fn calculate_strengths(crew: u32, gov: Government, weps: &[(Weapon, u32)]) -> Vec<PowerLevel> {
    let length = weps.iter().map(|x| x.1).fold(0, |a,b| a + b);
    let mut attack_strengths = Vec::with_capacity(length as usize);
    let mut defense_strengths = Vec::with_capacity(length as usize);
    for wep in weps {
        let strength = wep.0 + gov;
        for _ in 0 .. wep.1 {
            attack_strengths.push(strength.attack_strength);
            defense_strengths.push(strength.defense_strength);
        }
    }
    debug_assert_eq!(attack_strengths.len(), defense_strengths.len());
    while attack_strengths.len() < crew as usize {
        attack_strengths.push(gov.attack_strength);
        defense_strengths.push(gov.defense_strength);
    }
    // sort in descending order
    attack_strengths.sort_by(|a,b| b.partial_cmp(a).unwrap());
    defense_strengths.sort_by(|a,b| b.partial_cmp(a).unwrap());
    // convert to running sums
    for n in 1 .. attack_strengths.len() { attack_strengths[n] += attack_strengths[n-1]; }
    for n in 1 .. defense_strengths.len() { defense_strengths[n] += defense_strengths[n-1]; }
    attack_strengths.truncate(crew as usize);
    defense_strengths.truncate(crew as usize);
    attack_strengths.into_iter().zip(defense_strengths.into_iter()).map(|(a,d)| PowerLevel::new(a,d)).collect()
}

fn sub_attempt<R: Rng>(rng: &mut R, mut ucrew: u32, mut mcrew: u32,
                       player_attack_odds: &CaptureOdds,
                       player_defense_odds: &CaptureOdds,
                       auto_go: bool)
                       -> bool {
    while ucrew > 1 && mcrew > 0 {
        let uatk_odds = player_attack_odds.capture_odds(ucrew, mcrew);
        let udef_odds = player_defense_odds.capture_odds(mcrew, ucrew);
        let enemy_attacks = udef_odds > 0.5;
        let you_attack = uatk_odds > udef_odds || !enemy_attacks;
        debug_assert!(enemy_attacks || you_attack);
        let rounds = if auto_go { (ucrew / 5).max(1) } else { 1 };
        for _ in 0 .. rounds {
            if ucrew == 0 || mcrew == 0 { break }
            let upow = if you_attack {
                player_attack_odds.attacker_power(ucrew)
            }
            else {
                player_defense_odds.defender_power(ucrew)
            };
            let mpow = if enemy_attacks {
                player_defense_odds.attacker_power(mcrew)
            }
            else {
                player_attack_odds.defender_power(mcrew)
            };
            let tpow = upow + mpow;
            debug_assert!(tpow > 0.0);
            if rng.gen::<f64>() * tpow >= upow { ucrew -= 1 }
            else { mcrew -= 1 }
        }
    }
    mcrew == 0
}

fn attempt<R: Rng>(rng: &mut R, ucrew: u32, mcrew: u32,
                   player_attack_odds: &CaptureOdds,
                   player_defense_odds: &CaptureOdds)
                   -> (bool, bool) {
    (sub_attempt(rng, ucrew, mcrew, player_attack_odds, player_defense_odds, true),
     sub_attempt(rng, ucrew, mcrew, player_attack_odds, player_defense_odds, false))
}

fn thread_worker<F: FnMut() -> bool>(icount: usize, ucrew: u32, mcrew: u32,
                                     player_attack_odds: &CaptureOdds,
                                     player_defense_odds: &CaptureOdds,
                                     mut should_continue: F)
                                     -> (usize, usize, usize) {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut ucount = 0;
    let mut victories_by_auto = 0;
    let mut victories_by_uni = 0;
    while should_continue() {
        ucount += 1;
        for _ in 0 .. icount {
            let (auto, uni) = attempt(&mut rng, ucrew, mcrew,
                                      &player_attack_odds,
                                      &player_defense_odds);
            if auto { victories_by_auto += 1 }
            if uni { victories_by_uni += 1}
        }
    }
    (ucount, victories_by_auto, victories_by_uni)
}

fn proceed_with_invocation(invocation: &Invocation) {
    let ustrengths = calculate_strengths(invocation.ucrew, invocation.ugov, &invocation.uwep);
    let mstrengths = calculate_strengths(invocation.mcrew, invocation.mgov, &invocation.mwep);
    let player_attack_odds = CaptureOdds::new(&ustrengths, &mstrengths);
    let player_defense_odds = CaptureOdds::new(&mstrengths, &ustrengths);
    if invocation.verbose {
        println!(r#"Initial Conditions
------------------

Who  | Crew |     Attack |    Defense
---- | ---- | ---------- | ----------
You  | {:>4} | {:>10.1} | {:>10.1}
Them | {:>4} | {:>10.1} | {:>10.1}

victory odds: {:.1}%  
(casualties): {:.1}

defeat odds:  {:.1}%  
(casualties): {:.1}
"#,
             invocation.ucrew, ustrengths.last().unwrap().attack_strength, ustrengths.last().unwrap().defense_strength,
             invocation.mcrew, mstrengths.last().unwrap().attack_strength, mstrengths.last().unwrap().defense_strength,
             player_attack_odds.capture_odds(invocation.ucrew, invocation.mcrew) * 100.0,
             player_attack_odds.attacker_casualties(invocation.ucrew, invocation.mcrew),
             player_defense_odds.capture_odds(invocation.mcrew, invocation.ucrew) * 100.0,
             player_defense_odds.defender_casualties(invocation.mcrew, invocation.ucrew),
        );
    }
    std::mem::drop(ustrengths);
    std::mem::drop(mstrengths);
    if let None = invocation.ucount.checked_mul(invocation.icount) {
        panic!("Absurdly huge total iteration count!");
    }
    let (thread_work_counts, thread_victories_by_auto, thread_victories_by_uni)
    = if invocation.tcount > 1 || invocation.force_threaded {
        let icount = invocation.icount;
        let ucrew = invocation.ucrew;
        let mcrew = invocation.mcrew;
        let player_attack_odds = Arc::new(player_attack_odds);
        let player_defense_odds = Arc::new(player_defense_odds);
        let remaining_work_units = Arc::new(DoomsdayClock::new(invocation.ucount));
        let mut threads = Vec::with_capacity(invocation.tcount - 1);
        for n in 1 .. invocation.tcount {
            let player_attack_odds = player_attack_odds.clone();
            let player_defense_odds = player_defense_odds.clone();
            let remaining_work_units = remaining_work_units.clone();
            threads.push(std::thread::Builder::new()
                .name(format!("worker thread {}", n))
                .spawn(move || {
                    thread_worker(icount, ucrew, mcrew, player_attack_odds.as_ref(), player_defense_odds.as_ref(), || remaining_work_units.tick())
                }).unwrap());
        }
        let mut work_counts = Vec::with_capacity(invocation.tcount);
        let mut victories_by_auto = Vec::with_capacity(invocation.tcount);
        let mut victories_by_uni = Vec::with_capacity(invocation.tcount);
        let mut last = Instant::now();
        let (a,b,c) = thread_worker(icount, ucrew, mcrew, player_attack_odds.as_ref(), player_defense_odds.as_ref(), || { let now = Instant::now(); if now - last >= Duration::new(1,0) { last = now; remaining_work_units.tick_loudly() } else { remaining_work_units.tick() }});
        work_counts.push(a);
        victories_by_auto.push(b);
        victories_by_uni.push(c);
        for thread in threads.into_iter() {
            let (a,b,c) = match thread.join() {
                Ok(x) => x,
                Err(x) => {
                    panic!("A thread panicked, so will we!\n{:?}", x);
                },
            };
            work_counts.push(a);
            victories_by_auto.push(b);
            victories_by_uni.push(c);
        }
        (work_counts, victories_by_auto, victories_by_uni)
    }
    else {
        let mut it = 0..invocation.ucount;
        let (a,b,c) = thread_worker(invocation.icount,
                                    invocation.ucrew, invocation.mcrew,
                                    &player_attack_odds, &player_defense_odds,
                                    || it.next().is_some());
        (vec![a], vec![b], vec![c])
    };
    let total_work_count = thread_work_counts.iter().fold(0, |a,b| a+b) * invocation.icount;
    let total_victories_by_auto = thread_victories_by_auto.iter().fold(0, |a,b| a+b);
    let total_victories_by_uni = thread_victories_by_uni.iter().fold(0, |a,b| a+b);
    let auto_victory_rate = total_victories_by_auto as f64 * 100.0 / total_work_count as f64;
    let uni_victory_rate = total_victories_by_uni as f64 * 100.0 / total_work_count as f64;
    print!(r#"Results
-------

Number of trials: {:}  
Victory rate with auto-go: **{:.1}%**  
Victory rate with one-at-a-time: **{:.1}%**

"#, total_work_count, auto_victory_rate, uni_victory_rate,
    );
    if (auto_victory_rate - uni_victory_rate).abs() < 1.0 {
        if auto_victory_rate < 5.0 {
            println!("You're pretty screwed either way.");
        }
        else {
            println!("There's no significant difference either way.");
        }
    }
    else if auto_victory_rate < uni_victory_rate {
        println!("One-at-a-time would give you a significant advantage.");
    }
    else {
        println!("One-at-a-time would put you at a **disadvantage**.");
    }
    print!(r#"
Work Statistics
---------------

Thread | Units | Auto wins |  Uni wins
------ | ----- | --------- | ---------
"#);
    for n in 0 .. thread_work_counts.len() {
        if n == 0 { print!("main  ") } else { print!("{:<6}", n) }
        println!(" | {:>5} | {:>9} | {:>9}",
                 thread_work_counts[n], thread_victories_by_auto[n],
                 thread_victories_by_uni[n]);
    }
}

fn main() {
    let invocation = match get_invocation() {
        Some(x) => x,
        None => std::process::exit(1),
    };
    proceed_with_invocation(&invocation);
}
