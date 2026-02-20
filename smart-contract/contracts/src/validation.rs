 #![allow(dead_code)]
use soroban_sdk::String;
 
 pub fn non_empty(s: &String) -> bool {
     s.len() > 0
 }
 
 pub fn max_len(s: &String, max: u32) -> bool {
     s.len() <= max
 }
