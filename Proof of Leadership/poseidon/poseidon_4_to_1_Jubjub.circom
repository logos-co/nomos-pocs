//test
pragma circom 2.1.9;

include "poseidon_Jubjub_4_to_1_constants.circom";

template external_linear_layer_4_to_1() {

	signal input in[4];
	signal output out[4];


	out[3] <== in[0] + in[0] + in[0] + in[0] + in[0] + in[1] + in[1] + in[1] + in[1] + in[1] + in[1] + in[1] + in[2] + in[3] + in[3] + in[3];
	out[1] <== in[0] + in[0] + in[0] + in[0] + in[1] + in[1] + in[1] + in[1] + in[1] + in[1] + in[2] + in[3];
	out[0] <== in[0] + in[1] + in[1] + in[1] + in[2] + in[2] + in[2] + in[2] + in[2] + in[3] + in[3] + in[3] + in[3] + in[3] + in[3] + in[3];
	out[2] <== in[0] + in[1] + in[2] + in[2] + in[2] + in[2] + in[3] + in[3] + in[3] + in[3] + in[3] + in[3];
}

template internal_linear_layer_4_to_1() {
	signal input in[4];
	signal output out[4];

	out[0] <== in[0] + in[0] + in[1] + in[2] + in[3];
	out[1] <== in[1] + in[0] + in[1] + in[2] + in[3];
	out[2] <== in[2] + in[2] + in[2] + in[0] + in[1] + in[2] + in[3];
	out[3] <== in[3] + in[3] + in[3] + in[3] + in[3] + in[3] + in[3] + in[0] + in[1] + in[2] + in[3];
}

template sbox_4_to_1() { // ALPHA = 5
	signal input in;
	signal output out;

	signal in2;
	signal in4;

	in2 <== in*in;
	in4 <== in2 * in2;
	out <== in4 * in;
}

template ARC_4_to_1(index){
	signal input in;
	signal output out;

	var constants[88] = round_constant_4_to_1();

	out <== in + constants[index];
}


template partial_round_4_to_1(round_number) {
	signal input in[4];
	signal output out[4];

	component add_constant = ARC_4_to_1(round_number + 12);
	add_constant.in <== in[0];

	component exp = sbox_4_to_1();
	exp.in <== add_constant.out;

	component matrix = internal_linear_layer_4_to_1();
	matrix.in[0] <== exp.out;
	matrix.in[1] <== in[1];
	matrix.in[2] <== in[2];
	matrix.in[3] <== in[3];

	out[0] <== matrix.out[0];
	out[1] <== matrix.out[1];
	out[2] <== matrix.out[2];
	out[3] <== matrix.out[3];
}

template full_rounds_4_to_1(round_number){
	signal input in[4];
	signal output out[4];

	component add_constant[4];
	if(round_number < 4) {
		add_constant[0] = ARC_4_to_1(round_number*4);
		add_constant[1] = ARC_4_to_1(round_number*4 +1);
		add_constant[2] = ARC_4_to_1(round_number*4 +2);
		add_constant[3] = ARC_4_to_1(round_number*4 +3);
	} else {
		add_constant[0] = ARC_4_to_1((round_number - 60) * 4 + 72);
		add_constant[1] = ARC_4_to_1((round_number - 60) * 4 + 73);
		add_constant[2] = ARC_4_to_1((round_number - 60) * 4 + 74);
		add_constant[3] = ARC_4_to_1((round_number - 60) * 4 + 75);
	}
	add_constant[0].in <== in[0];
	add_constant[1].in <== in[1];
	add_constant[2].in <== in[2];
	add_constant[3].in <== in[3];

	component exp[4];
	exp[0] = sbox_4_to_1();
	exp[1] = sbox_4_to_1();
	exp[2] = sbox_4_to_1();
	exp[3] = sbox_4_to_1();
	exp[0].in <== add_constant[0].out;
	exp[1].in <== add_constant[1].out;
	exp[2].in <== add_constant[2].out;
	exp[3].in <== add_constant[3].out;

	component matrix = external_linear_layer_4_to_1();
	matrix.in[0] <== exp[0].out;
	matrix.in[1] <== exp[1].out;
	matrix.in[2] <== exp[2].out;
	matrix.in[3] <== exp[3].out;

	out[0] <== matrix.out[0];
	out[1] <== matrix.out[1];
	out[2] <== matrix.out[2];
	out[3] <== matrix.out[3];
}

template permutation_4_to_1(){
	signal input in[4];
	signal output out[4];

	component full_rounds_4_to_1[8];
	component partial_round_4_to_1s[56];
	component matrix = external_linear_layer_4_to_1();

	matrix.in[0] <== in[0];
	matrix.in[1] <== in[1];
	matrix.in[2] <== in[2];
	matrix.in[3] <== in[3];

	for(var i=0; i<64; i++){
		if(i < 4) {
			full_rounds_4_to_1[i] = full_rounds_4_to_1(i);
		} else {
			if(i<60) {
				partial_round_4_to_1s[i-4] = partial_round_4_to_1(i);
			} else {
				full_rounds_4_to_1[i-56] = full_rounds_4_to_1(i);
			}
		}
	}

	for(var i=0; i<4; i++){
		full_rounds_4_to_1[0].in[i] <== matrix.out[i];
	}

	for(var i=1; i<4; i++){
		for(var j=0; j<4; j++){
			full_rounds_4_to_1[i].in[j] <== full_rounds_4_to_1[i-1].out[j];
		}
	}

	for(var i=0; i<4; i++){
		partial_round_4_to_1s[0].in[i] <== full_rounds_4_to_1[3].out[i];
	}

	for(var i=1; i<56; i++){
		for(var j=0; j<4; j++){
			partial_round_4_to_1s[i].in[j] <== partial_round_4_to_1s[i-1].out[j];
		}
	}

	for(var i=0; i<4; i++){
		full_rounds_4_to_1[4].in[i] <== partial_round_4_to_1s[55].out[i];
	}

	for(var i=5; i<8; i++){
		for(var j=0; j<4; j++){
			full_rounds_4_to_1[i].in[j] <== full_rounds_4_to_1[i-1].out[j];
		}
	}

	for(var i=0; i<4; i++){
		out[i] <== full_rounds_4_to_1[7].out[i];
	}
}

template hash_4_to_1(){
	signal input in[4];
	signal output out;

	component perm = permutation_4_to_1();
	perm.in[0] <== in[0];
	perm.in[1] <== in[1];
	perm.in[2] <== in[2];
	perm.in[3] <== in[3];

	out <== in[0] + perm.out[0] + in[1] + perm.out[1] + in[2] + perm.out[2] + in[3] + perm.out[3];
}

//component main = hash_4_to_1();