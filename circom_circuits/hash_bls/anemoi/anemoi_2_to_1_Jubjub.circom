//test
pragma circom 2.1.9;

include "anemoi_Jubjub_2_to_1_constants.circom";

template ark_layer_2_to_1(round_number) {
	assert(round_number < 21);

	signal input in[2];
	signal output out[2];

	var C[21] = C_2_to_1();
	var D[21] = D_2_to_1();

	out[0] <== in[0] + C[round_number];
	out[1] <== in[1] + D[round_number];
}

template pow_alpha_2_to_1() { // ALPHA = 5
	signal input in;
	signal output out;

	signal in2;
	signal in4;

	in2 <== in*in;
	in4 <== in2 * in2;
	out <== in4 * in;
}

template mds_layer_2_to_1() {
	signal input in[2];
	signal output out[2];

	out[1] <== in[1] + in[0];
	out[0] <== in[0] + out[1];
}

template s_box_2_to_1() {
	signal input in[2];
	signal output out[2];


	//Calculation using open Flystel
	var x;
	var y;
	x = in[0];
	y = in[1];

	x = x - 7 * (y**2);
	y = y - (x**20974350070050476191779096203274386335076221000211055129041463479975432473805); //   1/ALPHA
	x = x + 7 * (y**2) + 14981678621464625851270783002338847382197300714436467949315331057125308909861; //DELTA
	out[0] <-- x;
	out[1] <-- y;


	//Verification using closed Flystel

	component pow = pow_alpha_2_to_1();
	pow.in <== in[1] - out[1];

	signal y2;
	signal v2;
	y2 <== in[1]*in[1];
	v2 <==out[1]*out[1];
	in[0] === pow.out + 7 * y2;
	out[0] === pow.out + 7 * v2 + 14981678621464625851270783002338847382197300714436467949315331057125308909861;// DELTA
}

template round_2_to_1(round_number) {
	signal input in[2];
	signal output out[2];

	component cst = ark_layer_2_to_1(round_number);
	component mds = mds_layer_2_to_1();
	component sbox = s_box_2_to_1();

	cst.in[0] <== in[0];
	cst.in[1] <== in[1];
	mds.in[0] <== cst.out[0];
	mds.in[1] <== cst.out[1];
	sbox.in[0] <== mds.out[0];
	sbox.in[1] <== mds.out[1];
	out[0] <== sbox.out[0];
	out[1] <== sbox.out[1];
}

template permutation_2_to_1(){
	signal input in[2];
	signal output out[2];

	component rounds[21];
	component mds = mds_layer_2_to_1();

	for(var i = 0; i<21; i++){	//21 rounds 
		rounds[i] = round_2_to_1(i);
		if(i==0){
			rounds[i].in[0] <== in[0];
			rounds[i].in[1] <== in[1];
		} else {
			rounds[i].in[0] <== rounds[i-1].out[0];
			rounds[i].in[1] <== rounds[i-1].out[1];
		}
	}
	mds.in[0] <== rounds[20].out[0];
	mds.in[1] <== rounds[20].out[1];
	out[0] <== mds.out[0];
	out[1] <== mds.out[1];
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