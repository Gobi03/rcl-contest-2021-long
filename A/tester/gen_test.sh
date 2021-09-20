for i in `seq 1000`
do
    python generator.py "$i" > "inputs/input_${i}.txt"
done
