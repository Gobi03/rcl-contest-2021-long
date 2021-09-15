#/bin/bash

set -eu

tester_dir="A/tester"

judge_file="${tester_dir}/judge.py"

cargo build --release
for test_num in `seq 100`
do
    echo "$test_num"
    input_file="${tester_dir}/inputs/input_${test_num}.txt"
    output_file="${tester_dir}/outputs/output_${test_num}.txt"

    # cargo run --bin a < "${input_file}" > "${output_file}"
    ../target/release/a < "${input_file}" > "${output_file}"

    python "${judge_file}" "${input_file}" "${output_file}"
done
