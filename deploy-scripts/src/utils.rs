pub mod utils {
    pub fn wait() {
        // Necessary for random instances where the 'UTXO' cannot be found
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
}
