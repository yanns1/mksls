#[cfg(test)]
pub mod tests {
    pub fn vec_are_equal<T: Eq>(v1: &Vec<T>, v2: &Vec<T>) -> bool {
        v1.len() == v2.len() && v1.iter().fold(true, |acc, el| acc && v2.contains(el))
    }
}
