use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Deserialize, Serialize)]
pub struct StartDecideFirstPlayerOutboundData {
    pub user_id: u64,
    // あとから送る seed_a に対応する hash を送信
    pub hash: u64,
}

#[derive(Deserialize, Serialize)]
pub struct StartDecideFirstPlayerReturnData {
    pub user_id: u64,
    pub seed_b: u64,
}

#[derive(Deserialize, Serialize)]
pub struct EndDecideFirstPlayerOutboundData {
    pub seed_a: u64,
    // 保持している seed_a と 受信した seed_b から first_player を導く
    pub first_player_user_id: u64,
}

#[derive(Deserialize, Serialize)]
pub struct EndDecideFirstPlayerReturnData {
    // 以下のチェックを行なって問題なければ accept: true
    // 受信した seed_a から 受信した hash が対応していること
    // 保持している seed_b と 受信した seed_a から導かれた first_player が送信されたものと一致する
    pub accept: bool,
}

#[derive(Default)]
pub struct DecideFirstPlayerSequence {
    is_player_a: bool,
    own_user_id: u64,
    opponent_user_id: Option<u64>,
    own_seed: u64,
    opponent_seed: Option<u64>,
    suggested_first_player_id: Option<u64>,
    accepted: bool,
    hash: Option<u64>,
    position: usize,
}

impl DecideFirstPlayerSequence {
    pub fn new(own_user_id: u64, own_seed: u64, is_player_a: bool) -> DecideFirstPlayerSequence {
        DecideFirstPlayerSequence {
            is_player_a,
            own_user_id,
            opponent_user_id: None,
            own_seed,
            opponent_seed: None,
            suggested_first_player_id: None,
            accepted: false,
            hash: None,
            position: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.accepted
    }

    pub fn get_question_json(&mut self) -> Result<String, String> {
        Ok(if self.is_player_a {
            match self.position {
                0 => {
                    let outbound_data = self
                        .create_start_outbound_data()
                        .map_err(|err| format!("creating start outbound data failed: {err}"))?;
                    serde_json::to_string(&outbound_data).unwrap()
                }
                1 => {
                    let outbound_data = self
                        .create_end_outbound_data()
                        .map_err(|err| format!("creating end outbound data failed: {err}"))?;
                    serde_json::to_string(&outbound_data).unwrap()
                }
                _ => "".to_owned(),
            }
        } else {
            match self.position {
                1 => {
                    let return_data = self
                        .create_start_return_data()
                        .map_err(|err| format!("creating start return data failed: {err}"))?;
                    serde_json::to_string(&return_data).unwrap()
                }
                2 => {
                    let return_data = self
                        .create_end_return_data()
                        .map_err(|err| format!("creating end return data failed: {err}"))?;
                    self.position += 1;
                    serde_json::to_string(&return_data).unwrap()
                }
                _ => "".to_owned(),
            }
        })
    }

    pub fn set_answer_json(&mut self, json: &str) -> Result<(), String> {
        if self.is_player_a {
            match self.position {
                0 => {
                    let return_data =
                        serde_json::from_str::<StartDecideFirstPlayerReturnData>(json)
                            .map_err(|err| err.to_string())?;
                    self.receive_start_return_data(&return_data)?;
                    self.position += 1;
                }
                1 => {
                    let return_data = serde_json::from_str::<EndDecideFirstPlayerReturnData>(json)
                        .map_err(|err| err.to_string())?;
                    if return_data.accept {
                        self.accepted = true;
                        self.position += 2;
                    }
                }
                _ => return Err(format!("sequence has invalid position: {}", self.position)),
            }
        } else {
            match self.position {
                0 => {
                    let outbound_data =
                        serde_json::from_str::<StartDecideFirstPlayerOutboundData>(json)
                            .map_err(|err| err.to_string())?;
                    self.receive_start_outbound_data(&outbound_data)?;
                    self.position += 1;
                }
                1 => {
                    let outbound_data =
                        serde_json::from_str::<EndDecideFirstPlayerOutboundData>(json)
                            .map_err(|err| err.to_string())?;
                    self.receive_end_outbound_data(&outbound_data)?;
                    self.position += 1;
                }
                _ => return Err(format!("sequence has invalid position: {}", self.position)),
            }
        }
        Ok(())
    }

    pub fn is_swap_required(&self) -> Result<bool, String> {
        let Some(first_player_id) = self.suggested_first_player_id else {
            return Err("sequence does not complete, suggested first player id not set".into());
        };
        Ok(first_player_id != self.own_user_id)
    }

    fn decide_first_player(&self) -> Result<u64, String> {
        if self.opponent_user_id.is_none() || self.opponent_seed.is_none() {
            return Err("not ready for decide first player".to_owned());
        }
        let first_player_is_player_a = SmallRng::seed_from_u64(self.own_seed).random_bool(0.5)
            != SmallRng::seed_from_u64(self.opponent_seed.unwrap()).random_bool(0.5);
        if (first_player_is_player_a && self.is_player_a)
            || (!first_player_is_player_a && !self.is_player_a)
        {
            return Ok(self.own_user_id);
        }
        Ok(self.opponent_user_id.unwrap())
    }

    fn receive_start_outbound_data(
        &mut self,
        start_outbound_data: &StartDecideFirstPlayerOutboundData,
    ) -> Result<(), String> {
        if self.is_player_a {
            return Err(
                "cannot receive start outbound data because it is not b player".to_string(),
            );
        }
        self.opponent_user_id = Some(start_outbound_data.user_id);
        self.hash = Some(start_outbound_data.hash);
        Ok(())
    }

    fn receive_start_return_data(
        &mut self,
        start_return_data: &StartDecideFirstPlayerReturnData,
    ) -> Result<(), String> {
        if !self.is_player_a {
            return Err(
                "cannot receive start outbound data because it is not a player".to_string(),
            );
        }
        self.opponent_user_id = Some(start_return_data.user_id);
        self.opponent_seed = Some(start_return_data.seed_b);
        Ok(())
    }

    fn receive_end_outbound_data(
        &mut self,
        end_outbound_data: &EndDecideFirstPlayerOutboundData,
    ) -> Result<(), String> {
        if self.is_player_a {
            return Err(
                "cannot receive start outbound data because it is not b player".to_string(),
            );
        }
        self.opponent_seed = Some(end_outbound_data.seed_a);
        self.suggested_first_player_id = Some(end_outbound_data.first_player_user_id);
        Ok(())
    }

    pub fn set_is_player_a(&mut self, is_player_a: bool) {
        self.is_player_a = is_player_a;
    }

    fn create_start_outbound_data(&mut self) -> Result<StartDecideFirstPlayerOutboundData, String> {
        if !self.is_player_a {
            return Err("cannot call method because it is not a player".to_owned());
        }
        let hash = self.get_player_a_hash()?;
        self.hash = Some(hash);
        Ok(StartDecideFirstPlayerOutboundData {
            user_id: self.own_user_id,
            hash,
        })
    }

    fn create_start_return_data(&mut self) -> Result<StartDecideFirstPlayerReturnData, String> {
        if self.is_player_a {
            return Err("cannot call method because it is not b player".to_owned());
        }
        Ok(StartDecideFirstPlayerReturnData {
            user_id: self.own_user_id,
            seed_b: self.own_seed,
        })
    }

    fn create_end_outbound_data(&mut self) -> Result<EndDecideFirstPlayerOutboundData, String> {
        if !self.is_player_a {
            return Err("cannot call method because it is not a player".to_owned());
        }
        let first_player_user_id = self.decide_first_player()?;
        self.suggested_first_player_id = Some(first_player_user_id);
        Ok(EndDecideFirstPlayerOutboundData {
            seed_a: self.own_seed,
            first_player_user_id,
        })
    }

    fn create_end_return_data(&mut self) -> Result<EndDecideFirstPlayerReturnData, String> {
        if self.is_player_a {
            return Err("cannot call method because it is not b player".to_owned());
        }
        let hash = self.get_player_a_hash()?;
        let first_player_user_id = self.decide_first_player()?;
        let accept = hash == self.hash.unwrap()
            && self.suggested_first_player_id.unwrap() == first_player_user_id;
        self.accepted = accept;
        Ok(EndDecideFirstPlayerReturnData { accept })
    }

    fn get_player_a_hash(&self) -> Result<u64, String> {
        let mut hasher = DefaultHasher::new();
        if self.is_player_a {
            self.own_seed.hash(&mut hasher);
        } else {
            self.opponent_seed
                .ok_or_else(|| "opponent seed not set".to_owned())?
                .hash(&mut hasher);
        }
        Ok(hasher.finish())
    }
}
