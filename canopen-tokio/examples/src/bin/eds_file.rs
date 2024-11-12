use std::time::Duration;

use can_socket::tokio::CanSocket;
use canopen_tokio::{dictionary, nmt::NmtCommand, CanOpenSocket};

fn main() {
    env_logger::builder()
        .filter_module(module_path!(), log::LevelFilter::Info)
        .parse_default_env()
        .init();

    dictionary::ObjectDirectory::load_from_content(
        1,
        r#"
[FileInfo]
CreatedBy=LHY
ModifiedBy=LHY
Description=1
CreationTime=11:05AM
CreationDate=09-30-2020
ModificationTime=11:05AM
ModificationDate=09-30-2020
FileName=ZLAC8015-V1.0.eds
FileVersion=0x01
FileRevision=0x01
EDSVersion=4.0

[DeviceInfo]
VendorName=CANopen
VendorNumber=0x00000100
ProductName=ZLIM42
ProductNumber=0x00000001
RevisionNumber=0x00000000
OrderCode=0000
BaudRate_10=0
BaudRate_20=0
BaudRate_50=1
BaudRate_125=1
BaudRate_250=1
BaudRate_500=1
BaudRate_800=0
BaudRate_1000=1
SimpleBootUpMaster=0
SimpleBootUpSlave=1
Granularity=8
DynamicChannelsSupported=0
CompactPDO=0
GroupMessaging=0
NrOfRXPDO=4
NrOfTXPDO=4
LSS_Supported=0

[DummyUsage]
Dummy0001=0
Dummy0002=1
Dummy0003=1
Dummy0004=1
Dummy0005=1
Dummy0006=1
Dummy0007=1

[Comments]
Lines=0

[MandatoryObjects]
SupportedObjects=3
1=0x1000
2=0x1001
3=0x1018

[1000]
ParameterName=Device Type
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=0x40192
PDOMapping=0
ObjFlags=0x0

[1001]
ParameterName=Error Register
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[1018]
ParameterName=Identity Object
ObjectType=0x9
SubNumber=4
ObjFlags=0x0

[1018sub0]
ParameterName=number of entries
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=3
PDOMapping=0
LowLimit=1
HighLimit=4
ObjFlags=0

[1018sub1]
ParameterName=Vendor ID
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=0x00000100
PDOMapping=0
ObjFlags=0

[1018sub2]
ParameterName=Product code
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=0x00000001
PDOMapping=0
ObjFlags=0

[1018sub3]
ParameterName=Revision number
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=0x00000000
PDOMapping=0
ObjFlags=0

[OptionalObjects]
SupportedObjects=44
1=0x1005
2=0x1009
3=0x100A
4=0x1014
5=0x1017
6=0x1200
7=0x1400
8=0x1401
9=0x1402
10=0x1403
11=0x1600
12=0x1601
13=0x1602
14=0x1603
15=0x1800
16=0x1801
17=0x1802
18=0x1803
19=0x1A00
20=0x1A01
21=0x1A02
22=0x1A03
23=0x603F
24=0x6040
25=0x6041
26=0x605A
27=0x605B
28=0x605C
29=0x605D
30=0x6060
31=0x6061
32=0x6064
33=0x606C
34=0x607A
35=0x607C
36=0x6081
37=0x6083
38=0x6084
39=0x6085
40=0x6098
41=0x6099
42=0x609A
43=0x60FF
44=0x6077

[1005]
ParameterName=SYNC COB ID
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0x80
PDOMapping=0
ObjFlags=0x0

[1009]
ParameterName=hardware version
ObjectType=0x7
DataType=0x0009
AccessType=const
DefaultValue=0x2608
PDOMapping=0
ObjFlags=0x0

[100A]
ParameterName=software version
ObjectType=0x7
DataType=0x0009
AccessType=const
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[1014]
ParameterName=Emergency COB ID
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x80
PDOMapping=0
ObjFlags=0x0

[1017]
ParameterName=Producer Heartbeat Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[1200]
ParameterName=Server SDO Parameter
ObjectType=0x9
SubNumber=3
ObjFlags=0x0

[1200sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=2
PDOMapping=0
ObjFlags=0

[1200sub1]
ParameterName=COB ID Client to Server
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=$NODEID+0x600
PDOMapping=0
ObjFlags=0

[1200sub2]
ParameterName=COB ID Server to Client
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=$NODEID+0x580
PDOMapping=0
ObjFlags=0

[1400]
ParameterName=Receive PDO 1 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1400sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1400sub1]
ParameterName=COB-ID used by RPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x200
PDOMapping=0
ObjFlags=0

[1400sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=255
PDOMapping=0
ObjFlags=0

[1400sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1400sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1400sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1401]
ParameterName=Receive PDO 2 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1401sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1401sub1]
ParameterName=COB-ID used by RPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x300
PDOMapping=0
ObjFlags=0

[1401sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=255
PDOMapping=0
ObjFlags=0

[1401sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1401sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1401sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1402]
ParameterName=Receive PDO 3 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1402sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1402sub1]
ParameterName=COB-ID used by RPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x400
PDOMapping=0
ObjFlags=0

[1402sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=255
PDOMapping=0
ObjFlags=0

[1402sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1402sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1402sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1403]
ParameterName=Receive PDO 4 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1403sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1403sub1]
ParameterName=COB-ID used by RPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x500
PDOMapping=0
ObjFlags=0

[1403sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=255
PDOMapping=0
ObjFlags=0

[1403sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1403sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1403sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1600]
ParameterName=Receive PDO 1 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1600sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1600sub1]
ParameterName=Object 1600_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0x60400010
PDOMapping=0
ObjFlags=0

[1600sub2]
ParameterName=Object 1600_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0x60600008
PDOMapping=0
ObjFlags=0

[1600sub3]
ParameterName=Object 1600_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1600sub4]
ParameterName=Object 1600_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1601]
ParameterName=Receive PDO 2 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1601sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1601sub1]
ParameterName=Object 1601_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1601sub2]
ParameterName=Object 1601_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1601sub3]
ParameterName=Object 1601_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1601sub4]
ParameterName=Object 1601_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1602]
ParameterName=Receive PDO 3 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1602sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1602sub1]
ParameterName=Object 1602_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1602sub2]
ParameterName=Object 1602_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1602sub3]
ParameterName=Object 1602_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1602sub4]
ParameterName=Object 1602_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1603]
ParameterName=Receive PDO 4 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1603sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1603sub1]
ParameterName=Object 1603_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1603sub2]
ParameterName=Object 1603_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1603sub3]
ParameterName=Object 1603_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1603sub4]
ParameterName=Object 1603_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1800]
ParameterName=Transmit PDO 1 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1800sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1800sub1]
ParameterName=COB-ID used by TPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x180
PDOMapping=0
ObjFlags=0

[1800sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=254
PDOMapping=0
ObjFlags=0

[1800sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=100
PDOMapping=0
ObjFlags=0

[1800sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1800sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=00
PDOMapping=0
ObjFlags=0

[1801]
ParameterName=Transmit PDO 2 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1801sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1801sub1]
ParameterName=COB-ID used by TPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x280
PDOMapping=0
ObjFlags=0

[1801sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=254
PDOMapping=0
ObjFlags=0

[1801sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=100
PDOMapping=0
ObjFlags=0

[1801sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1801sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1802]
ParameterName=Transmit PDO 3 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1802sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1802sub1]
ParameterName=COB-ID used by TPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x380
PDOMapping=0
ObjFlags=0

[1802sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=254
PDOMapping=0
ObjFlags=0

[1802sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=100
PDOMapping=0
ObjFlags=0

[1802sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1802sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1803]
ParameterName=Transmit PDO 4 Parameter
ObjectType=0x9
SubNumber=6
ObjFlags=0x0

[1803sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=5
PDOMapping=0
ObjFlags=0

[1803sub1]
ParameterName=COB-ID used by TPDO
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=$NODEID+0x480
PDOMapping=0
ObjFlags=0

[1803sub2]
ParameterName=Transmission Type
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=254
PDOMapping=0
ObjFlags=0

[1803sub3]
ParameterName=Inhibit Time
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=100
PDOMapping=0
ObjFlags=0

[1803sub4]
ParameterName=Compatibility Entry
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1803sub5]
ParameterName=Event Timer
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A00]
ParameterName=Transmit PDO 1 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1A00sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1A00sub1]
ParameterName=Object 1A00_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0x60410010
PDOMapping=0
ObjFlags=0

[1A00sub2]
ParameterName=Object 1A00_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A00sub3]
ParameterName=Object 1A00_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A00sub4]
ParameterName=Object 1A00_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A01]
ParameterName=Transmit PDO 2 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1A01sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1A01sub1]
ParameterName=Object 1A01_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A01sub2]
ParameterName=Object 1A01_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A01sub3]
ParameterName=Object 1A01_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A01sub4]
ParameterName=Object 1A01_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A02]
ParameterName=Transmit PDO 3 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1A02sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1A02sub1]
ParameterName=Object 1A02_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A02sub2]
ParameterName=Object 1A02_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A02sub3]
ParameterName=Object 1A02_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A02sub4]
ParameterName=Object 1A02_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A03]
ParameterName=Transmit PDO 4 Mapping
ObjectType=0x9
SubNumber=5
ObjFlags=0x0

[1A03sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=rw
DefaultValue=4
PDOMapping=0
ObjFlags=0

[1A03sub1]
ParameterName=Object 1A03_sub1
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A03sub2]
ParameterName=Object 1A03_sub2
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A03sub3]
ParameterName=Object 1A03_sub3
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[1A03sub4]
ParameterName=Object 1A03_sub4
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[603F]
ParameterName=Error Code
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[6040]
ParameterName=ControlWord
ObjectType=0x7
DataType=0x0006
AccessType=wo
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[6041]
ParameterName=StatusWord
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[605A]
ParameterName=Quick_stop_option_code
ObjectType=0x7
DataType=0x0003
AccessType=rw
DefaultValue=5
PDOMapping=0
ObjFlags=0x0

[605B]
ParameterName=Shutdown option code
ObjectType=0x7
DataType=0x0003
AccessType=rw
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[605C]
ParameterName=Disable operation option code
ObjectType=0x7
DataType=0x0003
AccessType=rw
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[605D]
ParameterName=Halt option code
ObjectType=0x7
DataType=0x0003
AccessType=rw
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[6060]
ParameterName=Mode of operation
ObjectType=0x7
DataType=0x0002
AccessType=wo
DefaultValue=3
PDOMapping=0
ObjFlags=0x0

[6061]
ParameterName=Mode of operation display
ObjectType=0x7
DataType=0x0002
AccessType=ro
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[6064]
ParameterName=Actual position
ObjectType=0x7
DataType=0x0004
AccessType=ro
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[606C]
ParameterName=Current speed
ObjectType=0x7
DataType=0x0004
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[607A]
ParameterName=Target position
ObjectType=0x7
DataType=0x0004
AccessType=rw
DefaultValue=5000
PDOMapping=0
ObjFlags=0x0

[607C]
ParameterName=home offset
ObjectType=0x7
DataType=0x0004
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[6081]
ParameterName=posmode speed
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=60
PDOMapping=0
ObjFlags=0x0

[6083]
ParameterName=Acceleration time
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=2000
PDOMapping=0
ObjFlags=0x0

[6084]
ParameterName=Deceleration time
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=2000
PDOMapping=0
ObjFlags=0x0

[6085]
ParameterName=Quick stop decel time
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=10
PDOMapping=0
ObjFlags=0x0

[6098]
ParameterName=homing method
ObjectType=0x7
DataType=0x0002
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[6099]
ParameterName=homing speed/search speed
ObjectType=0x9
SubNumber=3
ObjFlags=0x0

[6099sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0005
AccessType=ro
DefaultValue=2
PDOMapping=0
ObjFlags=0

[6099sub1]
ParameterName=homing speed
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=120
PDOMapping=0
ObjFlags=0

[6099sub2]
ParameterName=homing search speed
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=60
PDOMapping=0
ObjFlags=0

[609A]
ParameterName=homing accel/decel time
ObjectType=0x7
DataType=0x0007
AccessType=rw
DefaultValue=100
PDOMapping=0
ObjFlags=0x0

[60FF]
ParameterName=Target speed
ObjectType=0x7
DataType=0x0004
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[6077]
ParameterName=Motor current
ObjectType=0x7
DataType=0x0004
AccessType=ro
DefaultValue=0
PDOMapping=1
ObjFlags=0x0

[ManufacturerObjects]
SupportedObjects=28
1=0x2000
2=0x2001
3=0x2002
4=0x2003
5=0x2004
6=0x2005
7=0x2006
8=0x2007
9=0x2008
10=0x2009
11=0x200A
12=0x200B
13=0x200C
14=0x200E
15=0x2010
16=0x2011
17=0x2012
18=0x2030
19=0x2040
20=0x2041
21=0x204B
22=0x204C
23=0x2069
24=0x2088
25=0x2029
26=0x2014
27=0x2015
28=0x2026


[2000]
ParameterName=Slavenodes
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[2001]
ParameterName=MotorState
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2002]
ParameterName=OperatingSpeed
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2003]
ParameterName=InStateGroup
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2004]
ParameterName=OutStateGroup
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2005]
ParameterName=CurrentSets
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=3
PDOMapping=0
ObjFlags=0x0

[2006]
ParameterName=DivNums
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=10
PDOMapping=0
ObjFlags=0x0

[2007]
ParameterName=LockCurrent
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2008]
ParameterName=Initial speed
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2009]
ParameterName=CanBaud
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=1
PDOMapping=0
ObjFlags=0x0

[200A]
ParameterName=ChaoStop
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=500
PDOMapping=0
ObjFlags=0x0

[200B]
ParameterName=PdValid
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[200C]
ParameterName=Motor pole pairs
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=25
PDOMapping=0
ObjFlags=0x0

[200E]
ParameterName=StartSpeed
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=5
PDOMapping=0
ObjFlags=0x0

[2010]
ParameterName=FunReset
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2011]
ParameterName=ErrReset
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2012]
ParameterName=CurPulRest
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2030]
ParameterName=InOutIO
ObjectType=0x9
SubNumber=9
ObjFlags=0x0

[2030sub0]
ParameterName=HighestSubIndex
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=14
PDOMapping=0
ObjFlags=0

[2030sub1]
ParameterName=InLogicLevelGroup1
ObjectType=0x7
DataType=0x000b
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030sub2]
ParameterName=G1OfsetX0
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030sub3]
ParameterName=G1OfsetX1
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030sub4]
ParameterName=G1OfsetX2
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030sub5]
ParameterName=G1OfsetX3
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030subC]
ParameterName=OutLogicLevelGroup1
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030subD]
ParameterName=G1OfsetY0
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2030subE]
ParameterName=G1OfsetY1
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=0
PDOMapping=0
ObjFlags=0

[2040]
ParameterName=CurrentKp
ObjectType=0x7
DataType=0x0006
AccessType=rw
PDOMapping=0
ObjFlags=0x0

[2041]
ParameterName=CurrentKi
ObjectType=0x7
DataType=0x0006
AccessType=rw
PDOMapping=0
ObjFlags=0x0

[204B]
ParameterName=X0/X1InputFilterTime
ObjectType=0x7
DataType=0x0006
AccessType=rw
PDOMapping=0
ObjFlags=0x0

[204C]
ParameterName=X2/X3InputFilterTime
ObjectType=0x7
DataType=0x0006
AccessType=rw
PDOMapping=0
ObjFlags=0x0

[2069]
ParameterName=DCvoltage
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0

[2029]
ParameterName=Battery voltage
ObjectType=0x7
DataType=0x0006
AccessType=ro
DefaultValue=0
PDOMapping=0
ObjFlags=0x0


[2088]
ParameterName=Version
ObjectType=0x7
DataType=0x0006
AccessType=ro
PDOMapping=0
ObjFlags=0x0

[2014]
ParameterName=Rated current
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=300
PDOMapping=0
ObjFlags=0x0

[2015]
ParameterName=Maximum current
ObjectType=0x7
DataType=0x0006
AccessType=rw
DefaultValue=600
PDOMapping=0
ObjFlags=0x0

[2026]
ParameterName=Temperature object
SubNumber=3

[2026sub0]
ParameterName=High temperature object index
ObjectType=0x7
DataType=0x0006
AccessType=ro
PDOMapping=0
ObjFlags=0x0

[2026sub1]
ParameterName=Motor temperature
ObjectType=0x7
DataType=0x0003
DefaultValue=4
AccessType=ro
PDOMapping=1
ObjFlags=0x0

[2026sub2]
ParameterName=Driver temperature
ObjectType=0x7
DataType=0x0003
AccessType=ro
PDOMapping=1
ObjFlags=0x0

[2027]
ParameterName=Motor status
ObjectType=0x7
DataType=0x0003
AccessType=ro
PDOMapping=1
ObjFlags=0x0

[DynamicChannels]
NrOfSeg=0
		"#,
    )
    .expect("Valid eds");
}
