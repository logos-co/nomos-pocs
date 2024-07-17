#!/bin/bash

set -e

TAU=../../../keys/powersOfTau20_BLS_final.ptau

circom "$1.circom" --r1cs --wasm -p bls12381
cd "$1_js/"
node generate_witness.js "$1.wasm" ../input.json ../witness.wtns
cd ..
rm -R "$1_js/"
snarkjs groth16 setup "$1.r1cs" $TAU circuit_0000.zkey -v
snarkjs zkey contribute circuit_0000.zkey circuit_0001.zkey --name="1st Contributor Name" -e="entropy" -v
snarkjs zkey beacon circuit_0001.zkey "$1.zkey" 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2" -v
snarkjs zkey export verificationkey "$1.zkey" verification_key.json -v

rm circuit_0*




#for i in `seq 1 20`;
#do
#	../../../rapidsnark/package/bin/prover "$1.zkey" witness.wtns proof.json public.json
        snarkjs groth16 prove "$1.zkey" witness.wtns proof.json public.json
#done



#start=$(date +%s%6N)
#for i in `seq 1 100`;
#do
snarkjs groth16 prove "$1.zkey" witness.wtns proof.json public.json
#../../../rapidsnark/package/bin/prover "$1.zkey" witness.wtns proof.json public.json
#done


#end=$(date +%s%6N)



rm witness.wtns

snarkjs groth16 verify verification_key.json public.json proof.json

#temps=$((($end-$start)/100))
#echo "Temps de la preuve: $temps micro secondes "

rm "$1.zkey"
