use std::collections::HashMap;

use crate::{Weapon, Government};

lazy_static! {
    static ref WEAPONS: HashMap<&'static str, Weapon> = [
        ("Fragmentation Grenades", Weapon::new(1.3, 0.3)),
        ("Intrusion Countermeasures", Weapon::new(0.0, 60.0)),
        ("Korath Repeater Rifle", Weapon::new(1.6, 2.4)),
        ("Laser Rifle", Weapon::new(0.6, 0.8)),
        ("Nerve Gas", Weapon::new(2.8, 0.8)),
        ("Pug Biodefenses", Weapon::new(0.0, 250.0)),
        ("Pulse Rifle", Weapon::new(0.7, 1.0)),
        ("Security Station", Weapon::new(0.0, 3.4)),
        ("Tuning Rifle", Weapon::new(1.2, 1.8)),
    ].iter().cloned().collect();
}

#[derive(Debug)]
pub struct Invocation {
    pub ucrew: u32,
    pub mcrew: u32,
    pub ugov: Government,
    pub mgov: Government,
    pub uwep: Vec<(Weapon, u32)>,
    pub mwep: Vec<(Weapon, u32)>,
    pub ucount: usize,
    pub icount: usize,
    pub tcount: usize,
    pub verbose: bool,
    pub force_threaded: bool,
}

fn print_usage(autonym: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} options...", autonym);
    print!("{}", opts.usage(&brief));
    print!(r#"

Example invocation, in which a souped-up Bactrian is attacking a harrowed
World-Ship:

{} \
    -ucrew 461 -uwep "Pug Biodefenses x 150" -uwep "Nerve Gas x 461" \
    -uwep "Tuning Rifle x 245" \
    -mcrew 799 -mgov 1.4/2.6 -mwep "Korath Repeater Rifle x 150"

Here are the weapons I know about:
"#, autonym);
    let mut weps: Vec<(&&str, &Weapon)> = WEAPONS.iter().collect();
    weps.sort_by(|a,b| -> std::cmp::Ordering {
        a.0.partial_cmp(b.0).unwrap()
    });
    for wep in weps {
        println!("\t{} (attack {}, defense {})", wep.0, wep.1.attack_strength,
                 wep.1.defense_strength);
    }
}

fn parse_weps(s: Vec<String>) -> Result<Vec<(Weapon, u32)>, ()> {
    let mut ret = Vec::with_capacity(s.len());
    for s in s.iter() {
        let s: Vec<&str> = s.split(" x ").collect();
        if s.len() != 2 { return Err(()) }
        let wep = match WEAPONS.get(s[0]) {
            None => { eprintln!("Unknown weapon: {}", s[0]); return Err(()) },
            Some(x) => x,
        };
        let count = match s[1].parse() {
            Err(_) => return Err(()),
            Ok(x) => x,
        };
        ret.push((*wep, count));
    }
    Ok(ret)
}

pub fn get_invocation() -> Option<Invocation> {
    let args: Vec<String> = std::env::args().collect();
    let autonym = args[0].clone();
    let default_thread_count = num_cpus::get();
    let mut opts = getopts::Options::new();
    opts.long_only(true);
    opts.optopt("", "ucount", "Specify the number of work units to run. Default is '100'.", "COUNT");
    opts.optopt("", "icount", "Specify the number of iterations per work unit. Larger values are less granular but more efficient. Default is '100'.", "COUNT");
    opts.optopt("", "tcount", &format!("Specify the number of threads to use. Default is {}, which is how many CPUs this computer seems to have.", default_thread_count), "COUNT");
    opts.reqopt("", "ucrew", "Specify the starting crew quantity on the player's ship. Required.", "CREW");
    opts.reqopt("", "mcrew", "Specify the starting crew quantity on the enemy's ship. Required.", "CREW");
    opts.optopt("", "ugov", "Specify the player's government's intrinsic attack and defense strengths. The default is '1.0/2.0', the only value this will ever have in vanilla.", "ATTACK/DEFENSE");
    opts.optopt("", "mgov", "Specify the enemy government's intrinsic attack and defense strengths. The default is '1.0/2.0', the most common values. Alpha and Korath governments have higher values.", "ATTACK/DEFENSE");
    opts.optmulti("", "uwep", "Specify a type of weapon on the player's ship, e.g. 'Laser Rifle x 47'. This option may be specified more than once.", "WEAPON x COUNT");
    opts.optmulti("", "mwep", "Specify a type of weapon on the enemy's ship, e.g. 'Korath Repeater Rifle x 150'. This option may be specified more than once.", "WEAPON x COUNT");
    let matches = match opts.parse(&args[1..]) {
        Ok(x) => x,
        Err(x) => {
            eprintln!("{}", x.to_string());
            print_usage(&autonym, &opts);
            return None
        }
    };
    let ucrew = match matches.opt_str("ucrew").unwrap().parse() {
        Err(_) | Ok(0) => {
            eprintln!("ucrew value must be a positive integer");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let mcrew = match matches.opt_str("mcrew").unwrap().parse() {
        Err(_) | Ok(0) => {
            eprintln!("mcrew value must be a positive integer");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let ugov = match matches.opt_str("ugov").as_ref().map(|x| x.as_str()).unwrap_or("1/2").parse() {
        Err(_) => {
            eprintln!("ugov value must be two positive floats separated by /");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let mgov = match matches.opt_str("mgov").as_ref().map(|x| x.as_str()).unwrap_or("1/2").parse() {
        Err(_) => {
            eprintln!("mgov value must be two positive floats separated by /");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let uwep = match parse_weps(matches.opt_strs("uwep")) {
        Err(_) => {
            eprintln!("uwep values must be in the form of \"WEAPON x COUNT\".");
            eprintln!("Example: \"Laser Rifle x 47\"");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let mwep = match parse_weps(matches.opt_strs("mwep")) {
        Err(_) => {
            eprintln!("mwep values must be in the form of \"WEAPON x COUNT\".");
            eprintln!("Example: \"Korath Repeater Rifle x 150\"");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let ucount = match matches.opt_get_default("ucount", 100) {
        Err(_) | Ok(0) => {
            eprintln!("ucount value must be a positive integer");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let icount = match matches.opt_get_default("icount", 100) {
        Err(_) | Ok(0) => {
            eprintln!("icount value must be a positive integer");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    let tcount = match matches.opt_get_default("tcount", default_thread_count) {
        Err(_) | Ok(0) => {
            eprintln!("tcount value must be a positive integer");
            print_usage(&autonym, &opts);
            return None
        },
        Ok(x) => x,
    };
    Some(Invocation{
        ucrew, mcrew, ugov, mgov, uwep, mwep, ucount, icount, tcount,
        verbose: true, force_threaded: false,
    })
}

