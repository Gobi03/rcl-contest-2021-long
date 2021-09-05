#/bin/bash

set -eu

test_num="0"

tester_dir="A/tester"
input_file="${tester_dir}/input_${test_num}.txt"
output_file="${tester_dir}/output_${test_num}.txt"

judge_file="${tester_dir}/judge.py"

cargo run --bin a < ${input_file} > ${output_file}
python ${judge_file} ${input_file} ${output_file}
