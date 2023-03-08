# Automating the build and test process
echo 'Building Contracts and running Unit Tests'
forc test
echo 'Running Integration Tests'
cargo test