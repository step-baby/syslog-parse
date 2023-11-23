use crate::protocol::single_parse;
use chrono::{DateTime, Duration, Local};
use nom::bytes::complete::take_until;
use nom::error::{Error, ParseError};

const DEFAULT_MAX_SIZE: usize = 1000;

#[derive(Debug)]
pub struct MsgStream {
    pub last_msg: String,
    pub res: Vec<u8>,
    max_size: usize,
    last_parse_time: DateTime<Local>,
}

impl Default for MsgStream {
    fn default() -> Self {
        MsgStream {
            last_msg: "".to_string(),
            res: vec![],
            max_size: DEFAULT_MAX_SIZE,
            last_parse_time: Local::now(),
        }
    }
}

impl MsgStream {
    pub fn set_max_size(&mut self, size: usize) {
        self.max_size = size;
    }

    pub fn clear(&mut self) {
        self.last_msg.clear();
        self.res.clear();
    }

    pub fn multi_parse<'a, T: ParseError<&'a str>>(
        &mut self,
        value: Vec<u8>,
    ) -> Result<Vec<String>, T> {
        self.last_parse_time = Local::now();
        let next_msg = if self.last_msg.ends_with('<') {
            self.last_msg.pop();
            vec!["<".as_bytes().to_vec(), self.res.clone(), value].concat()
        } else {
            vec![self.res.clone(), value].concat()
        };
        let next_msg = String::from_utf8_lossy(&next_msg);
        let mut next_msg = next_msg.as_ref();
        let mut logs = Vec::new();
        loop {
            match take_until::<&str, &str, Error<&str>>("<")(next_msg) {
                Ok((res, msg)) => {
                    self.last_msg += msg;
                    next_msg = res;
                    if let Ok((_, msg)) = single_parse(next_msg) {
                        next_msg = msg.msg;
                        if !self.last_msg.is_empty() {
                            logs.push(self.last_msg.clone());
                            self.last_msg.clear();
                        }
                        self.last_msg += &msg.header;
                        if logs.len().ge(&self.max_size) {
                            self.res = next_msg.as_bytes().to_vec();
                            return Ok(logs);
                        }
                    } else {
                        let first = next_msg.chars().next().unwrap_or_default();
                        self.last_msg.push(first);
                        next_msg = &next_msg[1..next_msg.len()];
                    }
                }
                Err(_) => {
                    self.res = next_msg.as_bytes().to_vec();
                    return Ok(logs);
                }
            }
        }
    }
}

impl Iterator for MsgStream {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let now = Local::now();
        let next_msg = if self.last_msg.ends_with('<') {
            self.last_msg.pop();
            vec!["<".as_bytes().to_vec(), self.res.clone()].concat()
        } else {
            self.res.clone()
        };
        let next_msg = String::from_utf8_lossy(&next_msg);
        let mut next_msg = next_msg.as_ref();
        loop {
            match take_until::<&str, &str, Error<&str>>("<")(next_msg) {
                Ok((res, msg)) => {
                    self.last_msg += msg;
                    next_msg = res;
                    if let Ok((_, msg)) = single_parse(next_msg) {
                        next_msg = msg.msg;
                        if !self.last_msg.is_empty() {
                            let last_msg = self.last_msg.clone();
                            self.last_msg = msg.header;
                            return Some(last_msg);
                        }
                    } else {
                        let first = next_msg.chars().next().unwrap_or_default();
                        self.last_msg.push(first);
                        next_msg = &next_msg[1..next_msg.len()];
                    }
                }
                Err(_) => {
                    self.res = next_msg.as_bytes().to_vec();
                    break;
                }
            }
        }

        let interval = now.signed_duration_since(self.last_parse_time);
        if interval.ge(&Duration::milliseconds(500)) && !self.last_msg.is_empty() {
            let msg = format!("{}{}", self.last_msg, String::from_utf8_lossy(&self.res));
            self.clear();
            return Some(msg);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let mut parsed = MsgStream::default();
        let value = "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - "
            .as_bytes()
            .to_vec();
        let msg = parsed.multi_parse::<Error<&str>>(value).unwrap();
        assert_eq!(msg.len(), 0);

        let value = "hello world<11>1 2023-09-07T09:45:08.899092Z localhost <90>myprogram5424 42 1545121 - tcp 传输 syslog<11>1 2023-09-07T09:45:08.899092Z localhost <90>myprogram5424 42 1545121 - tcp 传输 syslog2323".as_bytes().to_vec();
        let msg = parsed.multi_parse::<Error<&str>>(value).unwrap();
        assert_eq!(msg.len(), 2);

        parsed.last_parse_time = Local::now()
            .checked_sub_signed(Duration::seconds(10))
            .unwrap();
        let msg = parsed.next().unwrap();
        assert_eq!(msg.len(), 96);
    }

    #[test]
    fn test_2() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =
            "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - "
                .to_string();
        parsed.res = "hello world<46>1 2023-09-13T11:15".as_bytes().to_vec();

        let value = ":47.697131Z localhost dygen 48747 185 - hello world<46>1 2023-09-13T11:15:47.707412Z localhost dygen 48747 186 - hello world<46>1 2023-09-13T11:15:47.719776Z localhost dygen 48747 187 - hello world<46>1 2023-09-13T11:15:47.731068Z localhost dygen 48747 188 - hello world<46>1 2023-09-13T11:15:47.741438Z localhost dygen 48747 189 - hello world<46>1 2023-09-13T11:15:47.752566Z localhost dygen 48747 190 - hello world<46>1 2023-09-13T11:15:47.765048Z localhost dygen 48747 191 - hello world<46>1 2023-09-13T11:15:47.775234Z localhost dygen 48747 192 - hello world<46>1 2023-09-13T11:15:47.786644Z localhost dygen 48747 193 - hello world<46>1 2023-09-13T11:15:47.796719Z localhost dygen 48747 194 - hello world<46>1 2023-09-13T11:15:47.808883Z localhost dygen 48747 195 - hello world<46>1 2023-09-13T11:15:47.819102Z localhost dygen 48747 196 - hello world<46>1 2023-09-13T11:15:47.830662Z localhost dygen 48747 197 - hello world<46>1 2023-09-13T11:15:47.842487Z localhost dygen 48747 198 - hello world<46>1 2023-09-13T11:15:4".as_bytes().to_vec();
        let msg = parsed.multi_parse::<Error<&str>>(value).unwrap();
        assert_eq!(msg.len(), 14);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_3() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =
            "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - "
                .to_string();
        parsed.res = "hello world<46>1 202".as_bytes().to_vec();
        parsed.set_max_size(10);

        let value = "3-09-13T11:36:20.127738Z localhost dygen 50113 199 - hello world<46>1 2023-09-13T11:36:20.139313Z localhost dygen 50113 200 - hello world<46>1 2023-09-13T11:36:20.151548Z localhost dygen 50113 201 - hello world<46>1 2023-09-13T11:36:20.163635Z localhost dygen 50113 202 - hello world<46>1 2023-09-13T11:36:20.174879Z localhost dygen 50113 203 - hello world<46>1 2023-09-13T11:36:20.18724Z localhost dygen 50113 204 - hello world<46>1 2023-09-13T11:36:20.198862Z localhost dygen 50113 205 - hello world<46>1 2023-09-13T11:36:20.20901Z localhost dygen 50113 206 - hello world<46>1 2023-09-13T11:36:20.221036Z localhost dygen 50113 207 - hello world<46>1 2023-09-13T11:36:20.232828Z localhost dygen 50113 208 - hello world<46>1 2023-09-13T11:36:20.245382Z localhost dygen 50113 209 - hello world<46>1 2023-09-13T11:36:20.255931Z localhost dygen 50113 210 - hello world<46>1 2023-09-13T11:36:20.266635Z localhost dygen 50113 211 - hello world<46>1 2023-09-13T11:36:20.278109Z localhost dygen 50113 212 - hello world<46>1 2023-09";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 10);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_4() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =  "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - hello world<"
            .to_string();
        parsed.res = "46>1 2023-09-13T11:49:59.72".as_bytes().to_vec();

        let value = "0778Z localhost dygen 51187 437 - hello world<46>1 2023-09-13T11:49:59.732237Z localhost dygen 51187 438 - hello world<46>1 2023-09-13T11:49:59.743385Z localhost dygen 51187 439 - hello world<46>1 2023-09-13T11:49:59.755518Z localhost dygen 51187 440 - hello world<46>1 2023-09-13T11:49:59.766871Z localhost dygen 51187 441 - hello world<46>1 2023-09-13T11:49:59.777432Z localhost dygen 51187 442 - hello world<46>1 2023-09-13T11:49:59.78831Z localhost dygen 51187 443 - hello world<46>1 2023-09-13T11:49:59.8Z localhost dygen 51187 444 - hello world<46>1 2023-09-13T11:49:59.81139Z localhost dygen 51187 445 - hello world<46>1 2023-09-13T11:49:59.821541Z localhost dygen 51187 446 - hello world<46>1 2023-09-13T11:49:59.833128Z localhost dygen 51187 447 - hello world<46>1 2023-09-13T11:49:59.845375Z localhost dygen 51187 448 - hello world<46>1 2023-09-13T11:49:59.857092Z localhost dygen 51187 449 - hello world<46>1 2023-09-13T11:49:59.869078Z localhost dygen 51187 450 - hello world<46>1 2023-09-13T11:49:59.881514Z";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 14);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_5() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =  "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - hello world".to_string();
        parsed.res = "<46>1 2023-09-13T12:14:23.376067Z localhost dygen 52828 "
            .as_bytes()
            .to_vec();

        let value = "746 - hello world<46>1 2023-09-13T12:14:23.388603Z localhost dygen 52828 747 - hello world<46>1 2023-09-13T12:14:23.398801Z localhost dygen 52828 748 - hello world<46>1 2023-09-13T12:14:23.40943Z localhost dygen 52828 749 - hello world<46>1 2023-09-13T12:14:23.421159Z localhost dygen 52828 750 - hello world<46>1 2023-09-13T12:14:23.43254Z localhost dygen 52828 751 - hello world<46>1 2023-09-13T12:14:23.444639Z localhost dygen 52828 752 - hello world<46>1 2023-09-13T12:14:23.455732Z localhost dygen 52828 753 - hello world<46>1 2023-09-13T12:14:23.467854Z localhost dygen 52828 754 - hello world<46>1 2023-09-13T12:14:23.478999Z localhost dygen 52828 755 - hello world<46>1 2023-09-13T12:14:23.489494Z localhost dygen 52828 756 - hello world<46>1 2023-09-13T12:14:23.501359Z localhost dygen 52828 757 - hello world<46>1 2023-09-13T12:14:23.512624Z localhost dygen 52828 758 - hello world<46>1 2023-09-13T12:14:23.524783Z localhost dygen 52828 759 - hello world<46>1 2023-09-13T12:14:23.536167Z localhost dygen 52828 760 ";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 14);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_6() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =  "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - hello world".to_string();
        parsed.res = "<46>1                                    "
            .as_bytes()
            .to_vec();

        let value = "2023-09-13T15:39:43.346956Z localhost dygen 60458 901 - hello world<46>1 2023-09-13T15:39:43.357974Z localhost dygen 60458 902 - hello world<46>1 2023-09-13T15:39:43.368058Z localhost dygen 60458 903 - hello world<46>1 2023-09-13T15:39:43.378762Z localhost dygen 60458 904 - hello world<46>1 2023-09-13T15:39:43.388876Z localhost dygen 60458 905 - hello world<46>1 2023-09-13T15:39:43.401149Z localhost dygen 60458 906 - hello world<46>1 2023-09-13T15:39:43.412786Z localhost dygen 60458 907 - hello world<46>1 2023-09-13T15:39:43.423378Z localhost dygen 60458 908 - hello world<46>1 2023-09-13T15:39:43.433474Z localhost dygen 60458 909 - hello world<46>1 2023-09-13T15:39:43.444026Z localhost dygen 60458 910 - hello world<46>1 2023-09-13T15:39:43.456378Z localhost dygen 60458 911 - hello world<46>1 2023-09-13T15:39:43.466604Z localhost dygen 60458 912 - hello world<46>1 2023-09-13T15:39:43.477399Z localhost dygen 60458 913 - hello world<46>1 2023-09-13T15:39:43.488605Z localhost dygen 60458 914 - hello world<46>1 20";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 14);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_7() {
        let mut parsed = MsgStream::default();
        parsed.last_msg =  "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - hello world".to_string();
        parsed.res = "<46>1 2023-09-14T03:06:27.889916Z localho"
            .as_bytes()
            .to_vec();

        let value = "st dygen 71326 29 - hello world<46>1 2023-09-14T03:06:27.900082Z localhost dygen 71326 30 - hello world<46>1 2023-09-14T03:06:27.912639Z localhost dygen 71326 31 - hello world<46>1 2023-09-14T03:06:27.923987Z localhost dygen 71326 32 - hello world<46>1 2023-09-14T03:06:27.934671Z localhost dygen 71326 33 - hello world<46>1 2023-09-14T03:06:27.947075Z localhost dygen 71326 34 - hello world<46>1 2023-09-14T03:06:27.95862Z localhost dygen 71326 35 - hello world<46>1 2023-09-14T03:06:27.970422Z localhost dygen 71326 36 - hello world<46>1 2023-09-14T03:06:27.982756Z localhost dygen 71326 37 - hello world<46>1 2023-09-14T03:06:27.99446Z localhost dygen 71326 38 - hello world<46>1 2023-09-14T03:06:28.005502Z localhost dygen 71326 39 - hello world<46>1 2023-09-14T03:06:28.016147Z localhost dygen 71326 40 - hello world<46>1 2023-09-14T03:06:28.027632Z localhost dygen 71326 41 - hello world<46>1 2023-09-14T03:06:28.03863Z localhost dygen 71326 42 - hello world<46>1 2023-09-14T03:06:28.049456Z localhost dygen 71326 43 -";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 15);
        assert_eq!(msg[0].len(), 86);
    }

    #[test]
    fn test_8() {
        let mut parsed = MsgStream::default();
        parsed.last_msg = "<11>1 2023-09-07T09:45:08.899012Z localhost <90>myprogram5424 42 9533322 - hello world".to_string();
        parsed.res = "<46>1 202".as_bytes().to_vec();

        let value = "3-09-14T03:06:30.292049Z localhost dygen 71326 241 - hello world<46>1 2023-09-14T03:06:30.302195Z localhost dygen 71326 242 - hello world<46>1 2023-09-14T03:06:30.314143Z localhost dygen 71326 243 - hello world<46>1 2023-09-14T03:06:30.325347Z localhost dygen 71326 244 - hello world<46>1 2023-09-14T03:06:30.336867Z localhost dygen 71326 245 - hello world<46>1 2023-09-14T03:06:30.3483Z localhost dygen 71326 246 - hello world<46>1 2023-09-14T03:06:30.3599Z localhost dygen 71326 247 - hello world<46>1 2023-09-14T03:06:30.371696Z localhost dygen 71326 248 - hello world<46>1 2023-09-14T03:06:30.383623Z localhost dygen 71326 249 - hello world<46>1 2023-09-14T03:06:30.394759Z localhost dygen 71326 250 - hello world<46>1 2023-09-14T03:06:30.40684Z localhost dygen 71326 251 - hello world<46>1 2023-09-14T03:06:30.418307Z localhost dygen 71326 252 - hello world<46>1 2023-09-14T03:06:30.430398Z localhost dygen 71326 253 - hello world<46>1 2023-09-14T03:06:30.441497Z localhost dygen 71326 254 - hello world<46>1 2023-09-14";
        let msg = parsed
            .multi_parse::<Error<&str>>(value.as_bytes().to_vec())
            .unwrap();
        assert_eq!(msg.len(), 14);
        assert_eq!(msg[0].len(), 86);
    }
}
