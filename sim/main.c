
// Declare the QISA functions

void __quantum__qis__x__body(void *qubit);
void __quantum__qis__h__body(void *qubit);
void __quantum__qis__cz__body(void *control, void *target);
void __quantum__qis__mresetz__body(void *qubit, void *result);
int  __quantum__qis__read_result__body(void *result);
void __quantum__rt__array_record_output(long long size, void* label);
void __quantum__rt__result_record_output(void *rid, void *label);


// Utiltiy functions

void CX(int control, int target) {
    __quantum__qis__h__body((void*)target);
    __quantum__qis__cz__body((void*)control, (void*)target);
    __quantum__qis__h__body((void*)target);
}

void H(int qubit) {
    __quantum__qis__h__body((void*)qubit);
}

int MResetZ(int qubit, int result) {
    __quantum__qis__mresetz__body((void*)qubit, (void*)result);
    return __quantum__qis__read_result__body((void*)result);
}

void RecordResult(int result) {
    __quantum__rt__result_record_output((void*)result, (void*)0);
}


// Program functions

void make_random_state() {
    for (int i = 0; i < 9; i++) {
        H(i);
    }
}

int measure_as_int() {
    int result = 0;
    for (int i = 0; i < 9; i++) {
        result = result | (MResetZ(i, i) << i);
    }
    return result;
}

void random_numbers_over_500() {
    int result = 0;
    while (result < 500) {
        make_random_state();
        result = measure_as_int();
    }
    for(int i = 8; i >= 0; i--) {
        RecordResult(i);
    }
}

void full_entangle() {
    H(0);
    CX(0, 1);
    CX(1, 2);
    CX(2, 3);
    CX(3, 4);
    CX(4, 5);
    CX(5, 6);
    CX(6, 7);
    CX(7, 8);
    for(int i = 8; i >= 0; i--) {
        MResetZ(i, i);
        RecordResult(i);
    }
}

// Define the entry point function
void ENTRYPOINT__main(void) {
    //full_entangle();
    random_numbers_over_500();

    return;
}
