use crate::power_level::PowerLevel;

pub struct CaptureOdds {
    attacker_strengths: Vec<f64>,
    defender_strengths: Vec<f64>,
    capture_odds: Vec<f64>,
    attacker_casualties: Vec<f64>,
    defender_casualties: Vec<f64>,
}

impl CaptureOdds {
    // see CaptureOdds::Calculate in CaptureOdds.cpp
    // it doesn't actually matter to me whether this logic is all *correct*,
    // only whether it exactly duplicates the game's logic
    pub fn new(attacker_strengths: &[PowerLevel],
               defender_strengths: &[PowerLevel])
               -> CaptureOdds {
        let attacker_strengths: Vec<f64> = attacker_strengths.iter().map(|x| x.attack_strength).collect();
        let defender_strengths: Vec<f64> = defender_strengths.iter().map(|x| x.defense_strength).collect();
        let vec_size = attacker_strengths.len() * defender_strengths.len();
        let mut capture_odds = Vec::with_capacity(vec_size);
        let mut attacker_casualties = Vec::with_capacity(vec_size);
        let mut defender_casualties = Vec::with_capacity(vec_size);
        // row 1 = single attacker, cannot attack
        for _ in 0..defender_strengths.len() {
            capture_odds.push(0.0);
            attacker_casualties.push(0.0);
            defender_casualties.push(0.0);
        }
        let mut up = 0; // the index of this column on the preview row
        for attacker_count in 2 .. attacker_strengths.len()+1 {
            let attack_strength = attacker_strengths[attacker_count-1];
            // column 1 = single defender
            let odds = attack_strength / (attack_strength + defender_strengths[0]);
            capture_odds.push(odds + (1.0 - odds) * capture_odds[up]);
            attacker_casualties.push((1.0 - odds) * (attacker_casualties[up] + 1.0));
            defender_casualties.push(odds + (1.0 - odds) * (defender_casualties[up]));
            up += 1;
            for defender_count in 2 .. defender_strengths.len()+1 {
                let odds = attack_strength / (attack_strength + defender_strengths[defender_count-1]);
                capture_odds.push(odds * capture_odds.last().unwrap() + (1.0 - odds) * capture_odds[up]);
                attacker_casualties.push(odds * attacker_casualties.last().unwrap() + (1.0 - odds) * (attacker_casualties[up] + 1.0));
                defender_casualties.push(odds * (defender_casualties.last().unwrap() + 1.0) + (1.0 - odds) * defender_casualties[up]);
                up += 1;
            }
        }
        assert_eq!(capture_odds.len(), vec_size);
        CaptureOdds {
            attacker_strengths,
            defender_strengths,
            capture_odds,
            attacker_casualties,
            defender_casualties,
        }
    }
    pub fn crew_counts_to_index(&self,
                                remaining_attackers: u32,
                                remaining_defenders: u32)
                                -> usize {
        debug_assert!(remaining_attackers > 0);
        debug_assert!(remaining_attackers as usize <= self.attacker_strengths.len());
        debug_assert!(remaining_defenders > 0);
        debug_assert!(remaining_defenders as usize <= self.defender_strengths.len());
        (remaining_attackers as usize - 1) * self.defender_strengths.len()
            + (remaining_defenders as usize - 1)
    }
    pub fn capture_odds(&self, remaining_attackers: u32,
                        remaining_defenders: u32)
                        -> f64 {
        let index = self.crew_counts_to_index(remaining_attackers,
                                              remaining_defenders);
        self.capture_odds[index]
    }
    pub fn attacker_casualties(&self, remaining_attackers: u32,
                               remaining_defenders: u32)
                               -> f64 {
        let index = self.crew_counts_to_index(remaining_attackers,
                                              remaining_defenders);
        self.attacker_casualties[index]
    }
    pub fn defender_casualties(&self, remaining_attackers: u32,
                               remaining_defenders: u32)
                               -> f64 {
        let index = self.crew_counts_to_index(remaining_attackers,
                                              remaining_defenders);
        self.defender_casualties[index]
    }
    pub fn attacker_power(&self, remaining_attackers: u32) -> f64 {
        self.attacker_strengths[remaining_attackers as usize - 1]
    }
    pub fn defender_power(&self, remaining_defenders: u32) -> f64 {
        self.defender_strengths[remaining_defenders as usize- 1]
    }
}
