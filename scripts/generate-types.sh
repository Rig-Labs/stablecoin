# Set the contracts folder path
contracts_path="./contracts"

# Loop through each contract folder and generate the types
for contract in $contracts_path/*; do
  if [ -d "$contract" ]; then
    contract_name=$(basename "$contract")
    abi_path="$contract/out/debug/${contract_name}-abi.json"
    types_path="./types/$contract_name"
    echo "Generating types for $contract_name"
    echo "ABI path: $abi_path"
    echo "Types path: $types_path"
    npx fuels typegen --inputs "$abi_path" -o "$types_path" --silent
  fi
done
