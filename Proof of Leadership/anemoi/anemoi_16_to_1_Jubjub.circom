//test
pragma circom 2.1.9;

include "anemoi_Jubjub_16_to_1_constants.circom";

template ark_layer_16_to_1(round_number) {
	assert(round_number < 10);

	signal input in[16];
	signal output out[16];

	var C[10][8] = C_16_to_1();
	var D[10][8] = D_16_to_1();

	for(var i=0; i<8; i++){
		out[i] <== in[i] + C[round_number][i];
		out[8+i] <== in[8+i] + D[round_number][i];
	}
}

template pow_alpha_16_to_1() { // ALPHA = 5
	signal input in;
	signal output out;

	signal in2;
	signal in4;

	in2 <== in*in;
	in4 <== in2 * in2;
	out <== in4 * in;
}

template mds_layer_16_to_1() {
	signal input in[16];
	signal output out[16];

	/* M_x= 	[1 2 3 5 7 8 8 9]
				[9 1 2 3 5 7 8 8]
				[8 9 1 2 3 5 7 8]
				[8 8 9 1 2 3 5 7]
				[7 8 8 9 1 2 3 5]
				[5 7 8 8 9 1 2 3]
				[3 5 7 8 8 9 1 2]
				[2 3 5 7 8 8 9 1] */

	signal x[8];
	signal y[8];

	x[0] <== 1*in[0] + 2*in[1] + 3*in[2] + 5*in[3] + 7*in[4] + 8*in[5] + 8*in[6] + 9*in[7];
	x[1] <== 9*in[0] + 1*in[1] + 2*in[2] + 3*in[3] + 5*in[4] + 7*in[5] + 8*in[6] + 8*in[7];
	x[2] <== 8*in[0] + 9*in[1] + 1*in[2] + 2*in[3] + 3*in[4] + 5*in[5] + 7*in[6] + 8*in[7];
	x[3] <== 8*in[0] + 8*in[1] + 9*in[2] + 1*in[3] + 2*in[4] + 3*in[5] + 5*in[6] + 7*in[7];
	x[4] <== 7*in[0] + 8*in[1] + 8*in[2] + 9*in[3] + 1*in[4] + 2*in[5] + 3*in[6] + 5*in[7];
	x[5] <== 5*in[0] + 7*in[1] + 8*in[2] + 8*in[3] + 9*in[4] + 1*in[5] + 2*in[6] + 3*in[7];
	x[6] <== 3*in[0] + 5*in[1] + 7*in[2] + 8*in[3] + 8*in[4] + 9*in[5] + 1*in[6] + 2*in[7];
	x[7] <== 2*in[0] + 3*in[1] + 5*in[2] + 7*in[3] + 8*in[4] + 8*in[5] + 9*in[6] + 1*in[7];

	y[0] <== 1*in[9] + 2*in[10] + 3*in[11] + 5*in[12] + 7*in[13] + 8*in[14] + 8*in[15] + 9*in[8];
	y[1] <== 9*in[9] + 1*in[10] + 2*in[11] + 3*in[12] + 5*in[13] + 7*in[14] + 8*in[15] + 8*in[8];
	y[2] <== 8*in[9] + 9*in[10] + 1*in[11] + 2*in[12] + 3*in[13] + 5*in[14] + 7*in[15] + 8*in[8];
	y[3] <== 8*in[9] + 8*in[10] + 9*in[11] + 1*in[12] + 2*in[13] + 3*in[14] + 5*in[15] + 7*in[8];
	y[4] <== 7*in[9] + 8*in[10] + 8*in[11] + 9*in[12] + 1*in[13] + 2*in[14] + 3*in[15] + 5*in[8];
	y[5] <== 5*in[9] + 7*in[10] + 8*in[11] + 8*in[12] + 9*in[13] + 1*in[14] + 2*in[15] + 3*in[8];
	y[6] <== 3*in[9] + 5*in[10] + 7*in[11] + 8*in[12] + 8*in[13] + 9*in[14] + 1*in[15] + 2*in[8];
	y[7] <== 2*in[9] + 3*in[10] + 5*in[11] + 7*in[12] + 8*in[13] + 8*in[14] + 9*in[15] + 1*in[8];

	for(var i=0; i<8; i++){
		out[8+i] <== x[i] + y[i];
		out[i] <== x[i] + out[8+i];
	}
}


template s_box_16_to_1() {
	signal input in[16];
	signal output out[16];


	//Computation using open Flystel
	var x[8];
	var y[8];

	for(var i=0; i<8; i++){
		x[i] = in[i];
		y[i] = in[8+i];
		x[i] = x[i] - 7 * (y[i]**2);
		y[i] = y[i] - (x[i]**20974350070050476191779096203274386335076221000211055129041463479975432473805);		// 1/ALPHA
		x[i] = x[i] + 7 * (y[i]**2) + 14981678621464625851270783002338847382197300714436467949315331057125308909861; // DELTA
		out[i] <-- x[i];
		out[8+i] <-- y[i];
	}

	//Verification using closed Flystel
	component pow[8];
	signal y2[8];
	signal v2[8];

	for(var i=0; i<8; i++){
		pow[i] = pow_alpha_16_to_1();
		pow[i].in <== in[8+i] - out[8+i];
		y2[i] <== in[8+i]*in[8+i];
		v2[i] <== out[8+i]*out[8+i];
		in[i] === pow[i].out + 7 * y2[i];
		out[i] === pow[i].out + 7 * v2[i] + 14981678621464625851270783002338847382197300714436467949315331057125308909861;// DELTA
	}
}

template round_16_to_1(round_number) {
	signal input in[16];
	signal output out[16];

	component cst = ark_layer_16_to_1(round_number);
	component mds = mds_layer_16_to_1();
	component sbox = s_box_16_to_1();

	for(var i=0; i<16; i++){
		cst.in[i] <== in[i];
	}
	for(var i=0; i<16; i++){
		mds.in[i] <== cst.out[i];
	}
	for(var i=0; i<16; i++){
		sbox.in[i] <== mds.out[i];
	}
	for(var i=0; i<16; i++){
		out[i] <== sbox.out[i];
	}
}

template permutation_16_to_1(){
	signal input in[16];
	signal output out[16];

	component rounds[10];
	component mds = mds_layer_16_to_1();

	for(var i = 0; i<10; i++){	//10 rounds 
		rounds[i] = round_16_to_1(i);
		if(i==0){
			for(var j=0; j<16; j++){
				rounds[i].in[j] <== in[j];
			}
		} else {
			for(var j=0; j<16; j++){
				rounds[i].in[j] <== rounds[i-1].out[j];
			}
		}
	}
	for(var i=0; i<16; i++){
		mds.in[i] <== rounds[9].out[i];
	}
	for(var i=0; i<16; i++){
		out[i] <== mds.out[i];
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