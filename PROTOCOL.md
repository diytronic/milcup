
	char bufcurdir[512];
	char txdbuf[512];
	char rxdbuf[512];
	BYTE bufcod[0x20000];
	BYTE bufram[0x8000];
	BYTE buf_hexstr[530];
	BYTE buf_data_hex[256];
	BYTE bl_hex,btype_hex;
	WORD wadr_offs_hex;
	DWORD dwadr_seg_hex,dwadr_lineoffs_hex;
	DWORD dwadrboot;
	int ilboot,ilcod;
	CString str;
	CEdit m_com;
	HANDLE hthread;
	DWORD ThreadId;
	BOOL fStartRun;

# By default seems working on 9600

const DWORD baud[4]= {9600,19200,57600,115200};

# Check if alive

Write 512 times zero byte
Repeat If unable to read 3 byte back
If unable to read 3 byte back - error
Else ok

# Set baud rate

const DWORD baud[4]= {9600,19200,57600,115200};

Write 'B<3bytes>0'
Read while reading 1 byte

Write 0xd
Read 3 bytes 
If not <0xd 0xa 0x3e> - Error

# Boot load

Write 'L'
Write 2 byte dwaddrboot
Write 0
Write 0x20
Write 2 byte ilboot
Write 2 byte 0

Total 9 bytes

Read 'L' ok, else error

Write ilboot bytes (bufram+dwadrboot)

Read 'K' ok, else error

Write 9 bytes 

	for(i=0;i<(ilboot>>3);i++)
	{
		txdbuf[0] = 'Y';
		txdbuf[1] = (dwadrboot+8*i) & 0xff;
		txdbuf[2] = ((dwadrboot+8*i)>>8) & 0xff;
		txdbuf[3] = 0;
		txdbuf[4] = 0x20;
		txdbuf[5] = 8;
		txdbuf[6] = 0;
		txdbuf[7] = 0;
		txdbuf[8] = 0;
		com.WriteBlock(txdbuf,9);

Read 10 bytes start from 'Y<8bit dwaddrboot>K'
If not equal dwaddrboot error

Write 'R'
Write 2 byte dwaddrboot 
Write 0
Write 0x20

Total 5

Read 'R' or error

Write 'I'
Read 12 bytes
const char id_str[13]= "1986BOOTUART";
				if(rxdbuf[j] != id_str[j])


# Run program 

Just send 'R' and check if it is returned back.

# Erase

Write 'E'
Sleep 1 sec
Read 9 bytes strat from 'E'

	adr = (BYTE)rxdbuf[1] + (((BYTE)rxdbuf[2])<<8) + (((BYTE)rxdbuf[3])<<16)
		+ (((BYTE)rxdbuf[4])<<24);
	data = (BYTE)rxdbuf[5] + (((BYTE)rxdbuf[6])<<8) + (((BYTE)rxdbuf[7])<<16)
		+ (((BYTE)rxdbuf[8])<<24);
	if((adr == 0x08020000)&&(data == 0xffffffff))
	{
  Erase done
  }

# Program

Write
'A'
0x00
0x00
0x00
0x08

Total 5 bytes

Read 0x08 or error

Write 'P'

	for(i=0;i<(ilcod>>8);i++)
	{
		com.WriteBlock(txdbuf,1);
		com.WriteBlock((LPSTR)(bufcod+(i<<8)),256);
		ks =0;
		for(j=0;j<256;j++)
			ks += bufcod[j+(i<<8)];
		if((!com.ReadBlock(rxdbuf,1))||((BYTE)rxdbuf[0]!=ks))
		{
			str = "ошибка обмена";
			InsertStrToList();		
			com.Close();
			return 0;
		}
		m_progress.SetPos(i+1);
	}

# Verify

	txdbuf[0] = 'A';
	txdbuf[1] = 0x00;
	txdbuf[2] = 0x00;
	txdbuf[3] = 0x00;
	txdbuf[4] = 0x08;
	com.WriteBlock(txdbuf,5);
	if((!com.ReadBlock(rxdbuf,1))||(rxdbuf[0]!=0x08))

	txdbuf[0] = 'V';
	for(i=0;i<(ilcod>>8);i++)
	{
		for(j=0;j<32;j++)
		{
			com.WriteBlock(txdbuf,1);
			if(!com.ReadBlock(rxdbuf,8))
			{
				str = "ошибка обмена";
				InsertStrToList();		
				com.Close();
				return 0;
			}
			for(k=0;k<8;k++)
			{
				if((BYTE)rxdbuf[k] != bufcod[k+(j<<3)+(i<<8)])
				{
					
					m_list.DeleteString(m_list.GetCount()-1);
					str.Format("Verify failed adr=0x%08x dataw=0x%02x datar=0x%02x",
						0x08000000+k+(j<<3)+(i<<8),bufcod[k+(j<<3)+(i<<8)],(BYTE)rxdbuf[k]);
					InsertStrToList();
					com.Close();
					return 0;
				}
			}
		}
		m_progress.SetPos(i+1);
	}

# Load firmware from controller

	txdbuf[0] = 'A';
	txdbuf[1] = 0x00;
	txdbuf[2] = 0x00;
	txdbuf[3] = 0x00;
	txdbuf[4] = 0x08;
	com.WriteBlock(txdbuf,5);
	if((!com.ReadBlock(rxdbuf,1))||(rxdbuf[0]!=0x08))

	txdbuf[0] = 'V';
	for(i=0;i<512;i++)
	{
		for(j=0;j<32;j++)
		{
			com.WriteBlock(txdbuf,1);
			if(!com.ReadBlock(rxdbuf,8))
			{
				str = "ошибка обмена";
				InsertStrToList();		
				com.Close();
				return 0;
			}
			for(k=0;k<8;k++)
				bufcod[k+(j<<3)+(i<<8)] = (BYTE)rxdbuf[k];
		}
		m_progress.SetPos(i+1);
	}
	CString strfn;
	CFile file;
	strfn = "\\1986VE9x.bin";
	strfn = bufcurdir + strfn;
	file.Open(strfn,CFile::modeCreate|CFile::modeWrite,NULL);
	file.Write(bufcod,sizeof(bufcod));
	file.Close();

# Read Hex Code

Fill bufcod with 0xff

While Read line

Check if between  0x08000000 and 0x08020000


Address of data write Extended Segment Address + Extended Linear Address + Record Load offset 

dwadr = dwadr_lineoffs_hex + dwadr_seg_hex + wadr_offs_hex;


# Read Boot code

Fill bufram with 0xff

While Read line
  GetDataHex

Finally find dwadrboot and ilboot

			for(i=0;i<sizeof(bufram);i++)
			{
				if(bufram[i] != 0xff)
					break;
			}
			dwadrboot = i;
			for(i=(sizeof(bufram)-1);i>=0;i--)
			{
				if(bufram[i] != 0xff)
					break;
			}
			ilboot = (i+8 - dwadrboot) & 0xfffffff8;
			return 1;
		}


# GetDataHex



