#include <Arduino.h>
// https://github.com/PowerBroker2/SerialTransfer
#include <SerialTransfer.h>

#define STSerial Serial

SerialTransfer txFer;

void echo_callback()
{
  // The packed attribute is necessary, both sides should not have padding
  struct __attribute__((packed)) ECHO_STRUCT
  {
    uint8_t u_8 = 0;
    int8_t i_8 = 0;
    uint16_t u_16 = 0;
    int16_t i_16 = 0;
    float_t f_32 = 0.0;
    u_int8_t u8_arr[6] = {0, 0, 0, 0, 0, 0};
  };

  ECHO_STRUCT echo_struct;
  txFer.rxObj(echo_struct);

  uint8_t send_size = txFer.txObj(echo_struct);
  txFer.sendData(send_size);
}
void different_messages_callback()
{
  struct __attribute__((packed)) ONE_NUMBER_STRUCT
  {
    int32_t num = 0;
  };

  struct __attribute__((packed)) TWO_NUMBERS_STRUCT
  {
    int16_t num1 = 0;
    int16_t num2 = 0;
  };

  ONE_NUMBER_STRUCT one_number_struct;
  TWO_NUMBERS_STRUCT two_numbers_struct;

  txFer.rxObj(two_numbers_struct);

  one_number_struct.num = two_numbers_struct.num1 + two_numbers_struct.num2;

  uint8_t send_size = txFer.txObj(one_number_struct);
  txFer.sendData(send_size, 2);
  send_size = txFer.txObj(two_numbers_struct);
  txFer.sendData(send_size, 1);
}

const functionPtr callbackArr[] = {echo_callback, different_messages_callback};

void setup()
{
  STSerial.begin(115200);
  configST myConfig;
  myConfig.debug = false;
  myConfig.callbacks = callbackArr;
  myConfig.callbacksLen = sizeof(callbackArr) / sizeof(functionPtr);
  txFer.begin(STSerial, myConfig);
}

void loop()
{
  txFer.tick();
}