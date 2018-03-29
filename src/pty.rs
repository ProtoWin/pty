
use redox_termios::{Termios, Winsize};


use std::io::{self, Write, Read};
use std::collections::VecDeque;


// pub struct Termios {
//     pub c_iflag: tcflag_t, // 输入模式标志，控制终端输入方式
//     pub c_oflag: tcflag_t, // 输出模式标志，控制终端输出方式
//     pub c_cflag: tcflag_t, // 控制模式标志，指定终端硬件控制信息
//     pub c_lflag: tcflag_t, // 本地模式标志，控制终端编辑功能
//     pub c_cc: [cc_t; 32]
// }

/**

termio: System V terminal driver interface
termios:
    https://linux.die.net/man/3/termios


c_iflag:
    IGNBRK 忽略BREAK键输入
    BRKINT 如果设置了IGNBRK，BREAK键的输入将被忽略，如果设置了BRKINT ，将产生SIGINT中断
    IGNPAR 忽略奇偶校验错误
    PARMRK 标识奇偶校验错误
    INPCK 允许输入奇偶校验
    ISTRIP 去除字符的第8个比特
    INLCR 将输入的NL（换行）转换成CR（回车）
    IGNCR 忽略输入的回车
    ICRNL 将输入的回车转化成换行（如果IGNCR未设置的情况下）
    IUCLC 将输入的大写字符转换成小写字符（非POSIX）
    IXON 允许输入时对XON/XOFF流进行控制
    IXANY 输入任何字符将重启停止的输出
    IXOFF 允许输入时对XON/XOFF流进行控制
    IMAXBEL 当输入队列满的时候开始响铃，Linux在使用该参数而是认为该参数总是已经设置

c_oflag:
    OPOST 处理后输出
    OLCUC 将输入的小写字符转换成大写字符（非POSIX）
    ONLCR 将输入的NL（换行）转换成CR（回车）及NL（换行）
    OCRNL 将输入的CR（回车）转换成NL（换行）
    ONOCR 第一行不输出回车符
    ONLRET 不输出回车符
    OFILL 发送填充字符以延迟终端输出
    OFDEL 以ASCII码的DEL作为填充字符，如果未设置该参数，填充字符将是NUL（‘/0’）（非POSIX）
    NLDLY 换行输出延时，可以取NL0（不延迟）或NL1（延迟0.1s）
    CRDLY 回车延迟，取值范围为：CR0、CR1、CR2和 CR3
    TABDLY 水平制表符输出延迟，取值范围为：TAB0、TAB1、TAB2和TAB3
    BSDLY 空格输出延迟，可以取BS0或BS1
    VTDLY 垂直制表符输出延迟，可以取VT0或VT1
    FFDLY 换页延迟，可以取FF0或FF1

c_cflag:
    CBAUD 波特率（4+1位）（非POSIX）
    CBAUDEX 附加波特率（1位）（非POSIX）
    CSIZE 字符长度，取值范围为CS5、CS6、CS7或CS8
    CSTOPB 设置两个停止位
    CREAD 使用接收器
    PARENB 使用奇偶校验
    PARODD 对输入使用奇偶校验，对输出使用偶校验
    HUPCL 关闭设备时挂起
    CLOCAL 忽略调制解调器线路状态
    CRTSCTS 使用RTS/CTS流控制

c_lflag:
    ISIG 当输入INTR、QUIT、SUSP或DSUSP时，产生相应的信号
    ICANON 使用标准输入模式
    XCASE 在ICANON和XCASE同时设置的情况下，终端只使用大写。如果只设置了XCASE，则输入字符将被转换为小写字符，除非字符使用了转义字符（非POSIX，且Linux不支持该参数）
    ECHO 显示输入字符
    ECHOE 如果ICANON同时设置，ERASE将删除输入的字符，WERASE将删除输入的单词
    ECHOK 如果ICANON同时设置，KILL将删除当前行
    ECHONL 如果ICANON同时设置，即使ECHO没有设置依然显示换行符
    ECHOPRT 如果ECHO和ICANON同时设置，将删除打印出的字符（非POSIX）
    TOSTOP 向后台输出发送SIGTTOU信号
**/

#[derive(Debug)]
pub struct Pty {
    id: usize,
    attr: Termios,
    winsize: Winsize,
    cooked: Vec<u8>,
    mosi: VecDeque<Vec<u8>>,
    miso: VecDeque<u8>,
}

impl Pty {
    pub fn new(id: usize) -> Self {
        Pty {
            id: id,
            attr: Termios::default(),
            winsize: Winsize::default(),
            cooked: Vec::new(),
            mosi: VecDeque::new(),
            miso: VecDeque::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    // fn tcgetattr
    pub fn attr(&self) -> Termios {
        self.attr
    }

    // fn tcsetattr
    pub fn set_attr(&mut self, attr: Termios) {
        self.attr = attr;
    }


    pub fn winsize(&self) -> Winsize {
        self.winsize
    }

    pub fn set_winsize(&mut self, winsize: Winsize) {
        self.winsize = winsize;
    }

    // fn cfmakeraw
    // Termios::make_raw()
}

impl Write for Pty {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use redox_termios::*;

        let ifl = self.attr.c_iflag;
        //let ofl = &self.attr.c_oflag;
        //let cfl = &self.attr.c_cflag;
        let lfl = self.attr.c_lflag;
        let cc = self.attr.c_cc;

        let is_cc = |b: u8, i: usize| -> bool {
            b != 0 && b == cc[i]
        };

        let inlcr = ifl & INLCR == INLCR;
        let igncr = ifl & IGNCR == IGNCR;
        let icrnl = ifl & ICRNL == ICRNL;

        let echo = lfl & ECHO == ECHO;
        let echoe = lfl & ECHOE == ECHOE;
        let echonl = lfl & ECHONL == ECHONL;
        let icanon = lfl & ICANON == ICANON;
        let isig = lfl & ISIG == ISIG;
        let iexten = lfl & IEXTEN == IEXTEN;
        let ixon = lfl & IXON == IXON;

        for &byte in buf.iter() {
            let mut b = byte;

            // Input tranlation
            if b == b'\n' {
                if inlcr {
                    b = b'\r';
                }
            } else if b == b'\r' {
                if igncr {
                    b = 0;
                } else if icrnl {
                    b = b'\n';
                }
            }

            // Link settings
            if icanon {
                if b == b'\n' {
                    if echo || echonl {
                        // self.output(&[b]);
                    }

                    self.cooked.push(b);
                    self.mosi.push_back(self.cooked.clone());
                    self.cooked.clear();

                    b = 0;
                }

                if is_cc(b, VEOF) {
                    self.mosi.push_back(self.cooked.clone());
                    self.cooked.clear();

                    b = 0;
                }

                if is_cc(b, VEOL) {
                    if echo {
                        // self.output(&[b]);
                    }

                    self.cooked.push(b);
                    self.mosi.push_back(self.cooked.clone());
                    self.cooked.clear();

                    b = 0;
                }

                if is_cc(b, VEOL2) {
                    if echo {
                        // self.output(&[b]);
                    }

                    self.cooked.push(b);
                    self.mosi.push_back(self.cooked.clone());
                    self.cooked.clear();

                    b = 0;
                }

                if is_cc(b, VERASE) {
                    if let Some(_c) = self.cooked.pop() {
                        if echoe {
                            // self.output(&[8, b' ', 8]);
                        }
                    }

                    b = 0;
                }

                if is_cc(b, VWERASE) && iexten {
                    println!("VWERASE");
                    b = 0;
                }

                if is_cc(b, VKILL) {
                    println!("VKILL");
                    b = 0;
                }

                if is_cc(b, VREPRINT) && iexten {
                    println!("VREPRINT");
                    b = 0;
                }
            }

            if isig {
                if is_cc(b, VINTR) {
                    // kill -9 pid -s SIGINT
                    b = 0;
                }

                if is_cc(b, VQUIT) {
                    println!("VQUIT");
                    // kill -9 pid -s SIGQUIT

                    b = 0;
                }

                if is_cc(b, VSUSP) {
                    println!("VSUSP");
                    // kill -9 pid -s SIGTSTP
                    b = 0;
                }
            }

            if ixon {
                if is_cc(b, VSTART) {
                    println!("VSTART");
                    b = 0;
                }

                if is_cc(b, VSTOP) {
                    println!("VSTOP");
                    b = 0;
                }
            }

            if is_cc(b, VLNEXT) && iexten {
                println!("VLNEXT");
                b = 0;
            }

            if is_cc(b, VDISCARD) && iexten {
                println!("VDISCARD");
                b = 0;
            }

            if b != 0 {
                if echo {
                    // self.output(&[b]);
                }
                self.cooked.push(b);
            }
        }

        if ! icanon && self.cooked.len() >= cc[VMIN] as usize {
            self.mosi.push_back(self.cooked.clone());
            self.cooked.clear();
        }
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // self.0.read(buf)

        Ok(0)
    }

    // #[inline]
    // unsafe fn initializer(&self) -> io::Initializer {
    //     io::Initializer::nop()
    // }
}

