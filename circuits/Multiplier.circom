template Multiplier() {
   signal input a;
   signal input b;
   // private input means that this input is not public and will not be revealed in the proof

   signal output c;

   c <== a*b;
 }

component main = Multiplier();