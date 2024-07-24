//test
pragma circom 2.1.9;

include "poseidon_Jubjub_2_to_1_constants.circom";

template linear_layer_2_to_1() {

	signal input in[2];
	signal output out[2];

	out[0] <== in[0] + in[0] + in[1];
	out[1] <== in[0] + in[1] + in[1];
}

template sbox_2_to_1() { // ALPHA = 5
	signal input in;
	signal output out;

	signal in2;
	signal in4;

	in2 <== in*in;
	in4 <== in2 * in2;
	out <== in4 * in;
}

template ARC_2_to_1(index){
	signal input in;
	signal output out;

	var constants[72] = round_constant_2_to_1();

	out <== in + constants[index];
}



template partial_round_2_to_1(round_number) {
	signal input in[2];
	signal output out[2];

	component add_constant = ARC_2_to_1(round_number + 4);
	add_constant.in <== in[0];

	component exp = sbox_2_to_1();
	exp.in <== add_constant.out;

	component matrix = linear_layer_2_to_1();
	matrix.in[0] <== exp.out;
	matrix.in[1] <== in[1];

	out[0] <== matrix.out[0];
	out[1] <== matrix.out[1];
}

template full_rounds_2_to_1(round_number){
	signal input in[2];
	signal output out[2];

	component add_constant[2];
	if(round_number < 4) {
		add_constant[0] = ARC_2_to_1(round_number*2);
		add_constant[1] = ARC_2_to_1(round_number*2 +1);
	} else {
		add_constant[0] = ARC_2_to_1((round_number - 60) * 2 + 64);
		add_constant[1] = ARC_2_to_1((round_number - 60) * 2 + 65);
	}
	add_constant[0].in <== in[0];
	add_constant[1].in <== in[1];

	component exp[2];
	exp[0] = sbox_2_to_1();
	exp[1] = sbox_2_to_1();
	exp[0].in <== add_constant[0].out;
	exp[1].in <== add_constant[1].out;

	component matrix = linear_layer_2_to_1();
	matrix.in[0] <== exp[0].out;
	matrix.in[1] <== exp[1].out;

	out[0] <== matrix.out[0];
	out[1] <== matrix.out[1];
}

template permutation_2_to_1(){
	signal input in[2];
	signal output out[2];

	component full_rounds_2_to_1[8];
	component partial_round_2_to_1s[56];
	component matrix = linear_layer_2_to_1();

	matrix.in[0] <== in[0];
	matrix.in[1] <== in[1];

	for(var i=0; i<64; i++){
		if(i < 4) {
			full_rounds_2_to_1[i] = full_rounds_2_to_1(i);
		} else {
			if(i<60) {
				partial_round_2_to_1s[i-4] = partial_round_2_to_1(i);
			} else {
				full_rounds_2_to_1[i-56] = full_rounds_2_to_1(i);
			}
		}
	}

	full_rounds_2_to_1[0].in[0] <== matrix.out[0];
	full_rounds_2_to_1[0].in[1] <== matrix.out[1];

	for(var i=1; i<4; i++){
		full_rounds_2_to_1[i].in[0] <== full_rounds_2_to_1[i-1].out[0];
		full_rounds_2_to_1[i].in[1] <== full_rounds_2_to_1[i-1].out[1];
	}

	partial_round_2_to_1s[0].in[0] <== full_rounds_2_to_1[3].out[0];
	partial_round_2_to_1s[0].in[1] <== full_rounds_2_to_1[3].out[1];

	for(var i=1; i<56; i++){
		partial_round_2_to_1s[i].in[0] <== partial_round_2_to_1s[i-1].out[0];
		partial_round_2_to_1s[i].in[1] <== partial_round_2_to_1s[i-1].out[1];
	}

	full_rounds_2_to_1[4].in[0] <== partial_round_2_to_1s[55].out[0];
	full_rounds_2_to_1[4].in[1] <== partial_round_2_to_1s[55].out[1];

	for(var i=5; i<8; i++){
		full_rounds_2_to_1[i].in[0] <== full_rounds_2_to_1[i-1].out[0];
		full_rounds_2_to_1[i].in[1] <== full_rounds_2_to_1[i-1].out[1];
	}

	out[0] <== full_rounds_2_to_1[7].out[0];
	out[1] <== full_rounds_2_to_1[7].out[1];
}

template hash_2_to_1(){
	signal input in[2];
	signal output out;

	component perm = permutation_2_to_1();
	perm.in[0] <== in[0];
	perm.in[1] <== in[1];

	out <== in[0] + perm.out[0] + in[1] + perm.out[1];
}

//component main = hash_2_to_1();