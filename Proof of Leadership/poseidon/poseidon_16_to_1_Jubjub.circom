//test
pragma circom 2.1.9;

include "poseidon_Jubjub_16_to_1_constants.circom";

template external_linear_layer_16_to_1() {

	signal input in[16];
	signal output out[16];

	// Matrix is 	[10 14  2  6  5  7  1  3  5  7  1  3  5  7  1  3]
	//				[ 8 12  2  2  4  6  1  1  4  6  1  1  4  6  1  1]
	//				[ 2  6 10 14  1  3  5  7  1  3  5  7  1  3  5  7]
	//				[ 2  2  8 12  1  1  4  6  1  1  4  6  1  1  4  6]
	//				[ 5  7  1  3 10 14  2  6  5  7  1  3  5  7  1  3]
	//				[ 4  6  1  1  8 12  2  2  4  6  1  1  4  6  1  1]
	//				[ 1  3  5  7  2  6 10 14  1  3  5  7  1  3  5  7]
	//				[ 1  1  4  6  2  2  8 12  1  1  4  6  1  1  4  6]
	//				[ 5  7  1  3  5  7  1  3 10 14  2  6  5  7  1  3]
	//				[ 4  6  1  1  4  6  1  1  8 12  2  2  4  6  1  1]
	//				[ 1  3  5  7  1  3  5  7  2  6 10 14  1  3  5  7]
	//				[ 1  1  4  6  1  1  4  6  2  2  8 12  1  1  4  6]
	//				[ 5  7  1  3  5  7  1  3  5  7  1  3 10 14  2  6]
	//				[ 4  6  1  1  4  6  1  1  4  6  1  1  8 12  2  2]
	//				[ 1  3  5  7  1  3  5  7  1  3  5  7  2  6 10 14]
	//				[ 1  1  4  6  1  1  4  6  1  1  4  6  2  2  8 12]

	out[0] <==	in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +in[7] +in[7] +
				in[8] +in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +in[11] +in[11] +
				in[12] +in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15] +in[15] +in[15];
			
	out[1] <==	in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +in[2] +
				in[3] +in[3] +
				in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +
				in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +
				in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15];
			
	out[2] <==	in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +in[5] +in[5] +
				in[6] +in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +in[9] +in[9] +
				in[10] +in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +in[13] +in[13] +
				in[14] +in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[3] <==	in[0] +in[0] +
				in[1] +in[1] +
				in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +
				in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +
				in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +
				in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[4] <==	in[0] +in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +in[3] +in[3] +
				in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +in[11] +in[11] +
				in[12] +in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15] +in[15] +in[15];
			
	out[5] <==	in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +
				in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +in[6] +
				in[7] +in[7] +
				in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +
				in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15];
			
	out[6] <==	in[0] +
				in[1] +in[1] +in[1] +
				in[2] +in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +in[9] +in[9] +
				in[10] +in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +in[13] +in[13] +
				in[14] +in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[7] <==	in[0] +
				in[1] +
				in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +in[4] +
				in[5] +in[5] +
				in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +
				in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +
				in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[8] <==	in[0] +in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +in[3] +in[3] +
				in[4] +in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +in[7] +in[7] +
				in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15] +in[15] +in[15];
			
	out[9] <==	in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +
				in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +
				in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +in[10] +
				in[11] +in[11] +
				in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +
				in[15];
			
	out[10] <==	in[0] +
				in[1] +in[1] +in[1] +
				in[2] +in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +in[5] +in[5] +
				in[6] +in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +in[13] +in[13] +
				in[14] +in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[11] <==	in[0] +
				in[1] +
				in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +
				in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +in[8] +
				in[9] +in[9] +
				in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +
				in[13] +
				in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[12] <==	in[0] +in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +in[3] +in[3] +
				in[4] +in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +in[7] +in[7] +
				in[8] +in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +in[11] +in[11] +
				in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[13] <==	in[0] +in[0] +in[0] +in[0] +
				in[1] +in[1] +in[1] +in[1] +in[1] +in[1] +
				in[2] +
				in[3] +
				in[4] +in[4] +in[4] +in[4] +
				in[5] +in[5] +in[5] +in[5] +in[5] +in[5] +
				in[6] +
				in[7] +
				in[8] +in[8] +in[8] +in[8] +
				in[9] +in[9] +in[9] +in[9] +in[9] +in[9] +
				in[10] +
				in[11] +
				in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +in[14] +
				in[15] +in[15];
			
	out[14] <==	in[0] +
				in[1] +in[1] +in[1] +
				in[2] +in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +in[5] +in[5] +
				in[6] +in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +in[9] +in[9] +
				in[10] +in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +in[12] +
				in[13] +in[13] +in[13] +in[13] +in[13] +in[13] +
				in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
			
	out[15] <==	in[0] +
				in[1] +
				in[2] +in[2] +in[2] +in[2] +
				in[3] +in[3] +in[3] +in[3] +in[3] +in[3] +
				in[4] +
				in[5] +
				in[6] +in[6] +in[6] +in[6] +
				in[7] +in[7] +in[7] +in[7] +in[7] +in[7] +
				in[8] +
				in[9] +
				in[10] +in[10] +in[10] +in[10] +
				in[11] +in[11] +in[11] +in[11] +in[11] +in[11] +
				in[12] +in[12] +
				in[13] +in[13] +
				in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +in[14] +
				in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15] +in[15];
}

template internal_linear_layer_16_to_1() {
	signal input in[16];
	signal output out[16];

	signal sum <== in[0] + in[1] + in[2] + in[3] + in[4] + in[5] + in[6] + in[7] + in[8] + in[9] + in[10] + in[11] + in[12] + in[13] + in[14] + in[15];


	out[0] <== in[0] * 67 + sum;
	out[1] <== in[1] * 84 + sum;
	out[2] <== in[2] * 80 + sum;
	out[3] <== in[3] * 94 + sum;
	out[4] <== in[4] * 57 + sum;
	out[5] <== in[5] * 89 + sum;
	out[6] <== in[6] * 92 + sum;
	out[7] <== in[7] * 39 + sum;
	out[8] <== in[8] * 34 + sum;
	out[9] <== in[9] * 24 + sum;
	out[10] <== in[10] + sum;
	out[11] <== in[11] * 95 + sum;
	out[12] <== in[12] * 21 + sum;
	out[13] <== in[13] * 73 + sum;
	out[14] <== in[14] * 68 + sum;
	out[15] <== in[15] * 52 + sum;

}

template sbox_16_to_1() { // ALPHA = 5
	signal input in;
	signal output out;

	signal in2;
	signal in4;

	in2 <== in*in;
	in4 <== in2 * in2;
	out <== in4 * in;
}

template ARC_16_to_1(index){
	signal input in;
	signal output out;

	var constants[185] = round_constant_16_to_1();

	out <== in + constants[index];
}


template partial_round_16_to_1(round_number) {
	signal input in[16];
	signal output out[16];

	component add_constant = ARC_16_to_1(round_number + 60);
	add_constant.in <== in[0];

	component exp = sbox_16_to_1();
	exp.in <== add_constant.out;

	component matrix = internal_linear_layer_16_to_1();
	matrix.in[0] <== exp.out;
	for(var i=1; i<16; i++){
		matrix.in[i] <== in[i];
	}
	for(var i=0; i<16; i++){
		out[i] <== matrix.out[i];
	}
}

template full_rounds_16_to_1(round_number){
	signal input in[16];
	signal output out[16];

	component add_constant[16];
	if(round_number < 4) {
		for(var i=0; i<16; i++){
			add_constant[i] = ARC_16_to_1(round_number*16+i);
		}
	} else {
		for(var i=0; i<16; i++){
			add_constant[i] = ARC_16_to_1((round_number - 61) * 16 + i + 121);
		}
	}
	for(var i=0; i<16; i++){
		add_constant[i].in <== in[i];
	}

	component exp[16];
	for(var i=0; i<16; i++){
		exp[i] = sbox_16_to_1();
		exp[i].in <== add_constant[i].out;
	}

	component matrix = external_linear_layer_16_to_1();
	for(var i=0; i<16; i++){
		matrix.in[i] <== exp[i].out;
	}
	for(var i=0; i<16; i++){
		out[i] <== matrix.out[i];
	}
}

template permutation_16_to_1(){
	signal input in[16];
	signal output out[16];

	component full_rounds_16_to_1[8];
	component partial_round_16_to_1s[57];
	component matrix = external_linear_layer_16_to_1();

	for(var i=0; i<16; i++){
		matrix.in[i] <== in[i];
	}

	for(var i=0; i<65; i++){
		if(i < 4) {
			full_rounds_16_to_1[i] = full_rounds_16_to_1(i);
		} else {
			if(i<61) {
				partial_round_16_to_1s[i-4] = partial_round_16_to_1(i);
			} else {
				full_rounds_16_to_1[i-57] = full_rounds_16_to_1(i);
			}
		}
	}

	for(var i=0; i<16; i++){
		full_rounds_16_to_1[0].in[i] <== matrix.out[i];
	}

	for(var i=1; i<4; i++){
		for(var j=0; j<16; j++){
			full_rounds_16_to_1[i].in[j] <== full_rounds_16_to_1[i-1].out[j];
		}
	}

	for(var i=0; i<16; i++){
		partial_round_16_to_1s[0].in[i] <== full_rounds_16_to_1[3].out[i];
	}

	for(var i=1; i<57; i++){
		for(var j=0; j<16; j++){
			partial_round_16_to_1s[i].in[j] <== partial_round_16_to_1s[i-1].out[j];
		}
	}

	for(var i=0; i<16; i++){
		full_rounds_16_to_1[4].in[i] <== partial_round_16_to_1s[56].out[i];
	}

	for(var i=5; i<8; i++){
		for(var j=0; j<16; j++){
			full_rounds_16_to_1[i].in[j] <== full_rounds_16_to_1[i-1].out[j];
		}
	}

	for(var i=0; i<16; i++){
		out[i] <== full_rounds_16_to_1[7].out[i];
	}
}

template hash_16_to_1(){
	signal input in[16];
	signal output out;

	component perm = permutation_16_to_1();
	for(var i=0; i<16; i++){
		perm.in[i] <== in[i];
	}

	out <== in[0] + perm.out[0] +
			in[1] + perm.out[1] +
			in[2] + perm.out[2] +
			in[3] + perm.out[3] +
			in[4] + perm.out[4] +
			in[5] + perm.out[5] +
			in[6] + perm.out[6] +
			in[7] + perm.out[7] +
			in[8] + perm.out[8] +
			in[9] + perm.out[9] +
			in[10] + perm.out[10] +
			in[11] + perm.out[11] +
			in[12] + perm.out[12] +
			in[13] + perm.out[13] +
			in[14] + perm.out[14] +
			in[15] + perm.out[15];
}

//component main = hash_16_to_1();