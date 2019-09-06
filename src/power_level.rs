use std::str::FromStr;

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct PowerLevel {
    pub attack_strength: f64,
    pub defense_strength: f64,
}

impl PowerLevel {
    pub fn new(attack_strength: f64, defense_strength: f64) -> PowerLevel {
        PowerLevel { attack_strength, defense_strength }
    }
}

impl std::ops::Add for PowerLevel {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        PowerLevel {
            attack_strength: self.attack_strength + rhs.attack_strength,
            defense_strength: self.defense_strength + rhs.defense_strength,
        }
    }
}

impl FromStr for PowerLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.split('/').collect();
        if s.len() != 2 { return Err(()) }
        let attack_strength = match s[0].parse() {
            Err(_) => return Err(()),
            Ok(x) if x <= 0.0 => return Err(()),
            Ok(x) => x,
        };
        let defense_strength = match s[1].parse() {
            Err(_) => return Err(()),
            Ok(x) if x <= 0.0 => return Err(()),
            Ok(x) => x,
        };
        Ok(Self::new(attack_strength, defense_strength))
    }
}

