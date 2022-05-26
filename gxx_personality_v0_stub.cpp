#include "unwind.h"
#include "stdint.h"

extern "C" _Unwind_Reason_Code __gxx_personality_v0   (int version, _Unwind_Action actions, uint64_t exceptionClass, _Unwind_Exception* unwind_exception, _Unwind_Context* context) {
    return  _URC_NO_REASON;
}