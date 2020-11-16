use ncurses::*;
use crate::virpc;
use crate::virpc::cpu;

static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_KEYWORD: i16 = 2;
static COLOR_PAIR_CURRENT: i16 = 3;
static MEMORY_SIZE: u32 = 0x080000;

//TODO sound chip
//TODO keyboard
//TODO video-improve
//TODO interrupt?
//TODO disk?

//Optional:
//TODO add custom handling of ldr/str arguments, and argument printing
//TODO add whole memview(separate window)
//TODO add colorised modified values in hexview
//TODO add help: arrows to navigate, tab to switch, e to edit in hex view, f to find(in code/hex view)

static EBCDIC: [char;256] = [
    /* 0   1   2   3   4   5   6   7   8   9   A   B   C   D    E   F */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* 0 */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* 1 */
      ' ','!','"','#','$','%','&','\'','(',')','*','+',',','-','.','/', /* 2 */
      '0','1','2','3','4','5','6','7','8','9',':',';','<','=' ,'>','?', /* 3 */
      '@','A','B','C','D','E','F','G','H','I','J','K','L','M' ,'N','O', /* 4 */
      'P','Q','R','S','T','U','V','W','X','Y','Z','[','\\',']','^','_', /* 5 */
      '`','a','b','c','d','e','f','g','h','i','j','k','l','m' ,'n','o', /* 6 */
      'p','q','r','s','t','u','v','w','x','y','z','{','|','}' ,'~','.', /* 7 */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* 8 */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* 9 */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* A */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* B */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* C */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* D */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.', /* E */
      '.','.','.','.','.','.','.','.','.','.','.','.','.','.' ,'.','.'];/* F */


pub struct Windows {
    items1 : Vec<ITEM>,
    items2 : Vec<ITEM>,
    items3 : Vec<ITEM>,
    menu1 : MENU,
    menu2 : MENU,
    menu3 : MENU,
    win2_sub : WINDOW,
    win1 : WINDOW,
    win2 : WINDOW,
    win3 : WINDOW,
    win4 : WINDOW,
    win5 : WINDOW,
    screen_height : i32,
    screen_width : i32,
    screen_height_n : i32,
    screen_width_n : i32,
    focus : i32,
    edit_line : u32,
    edit_pc : u32,
    current_pc : u32,
    menu1_choice : u32,
    menu2_choice : u32,
    menu3_choice : u32,
    cpu_reader : cpu::CPUShared,
    cur_arg : u32,
    edit_cmd : i32,
    edit_item : Vec<i32>,
    edit_mode : Vec<i32>,
    hex_view_offset : u32,
    mem_address : u32,
    mem_highlight : u32,
    mem_highlight_size : u32,
    virpc : virpc::Virpc,
    run_program : bool,
}

impl Windows {
    pub fn new(cpu : cpu::CPUShared, virpc : virpc::Virpc) -> Windows {

        let mut win = Windows {
            menu1 : 0 as MENU,
            menu2 : 0 as MENU,
            menu3 : 0 as MENU,
            items1 : Vec::new(),
            items2 : Vec::new(),
            items3 : Vec::new(),
            win2_sub: 0 as WINDOW,
            win1 : 0 as WINDOW,
            win2 : 0 as WINDOW,
            win3 : 0 as WINDOW,
            win4 : 0 as WINDOW,
            win5 : 0 as WINDOW,
            screen_height : 0,
            screen_width : 0,
            screen_height_n : 0,
            screen_width_n : 0,
            focus : 0,
            edit_line : 0,
            edit_pc : 0,
            current_pc : 0,
            menu1_choice : 0,
            menu2_choice : 0,
            menu3_choice : 0,
            cpu_reader :  cpu,
            cur_arg : 0,
            edit_cmd : -1,
            edit_item : vec![-1; 3],
            edit_mode : vec![-1; 3],
            hex_view_offset : 0,
            mem_address : 0,
            mem_highlight : 0,
            mem_highlight_size : 0,
            virpc : virpc,
            run_program : false,
        };

        initscr();
        keypad(stdscr(), true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        start_color();
        init_pair(COLOR_PAIR_DEFAULT, COLOR_WHITE, COLOR_BLACK);
        init_pair(COLOR_PAIR_KEYWORD, COLOR_BLACK, COLOR_WHITE);
        init_pair(COLOR_PAIR_CURRENT, COLOR_WHITE, COLOR_GREEN);

        refresh();//needed for screen size
        getmaxyx(stdscr(), &mut win.screen_height, &mut win.screen_width);

        win.win1 = newwin(win.wd(1,'h'), win.wd(1,'w'), win.wd(1,'y'), win.wd(1,'x'));
        win.win2 = newwin(win.wd(2,'h'), win.wd(2,'w'), win.wd(2,'y'), win.wd(2,'x'));
        win.win3 = newwin(win.wd(3,'h'), win.wd(3,'w'), win.wd(3,'y'), win.wd(3,'x'));
        win.win4 = newwin(win.wd(4,'h'), win.wd(4,'w'), win.wd(4,'y'), win.wd(4,'x'));
        win.win2_sub = derwin(win.win2,win.wd(2,'h')-2,win.wd(2,'w')-2,1,1);
        win.win5 = newwin(win.wd(5,'h'), win.wd(5,'w'), win.wd(5,'y'), win.wd(5,'x'));

        win.items1 = win.cpu_reader.borrow_mut().get_commands_list();
        win.items2 = win.cpu_reader.borrow_mut().get_data_list();
        win.items3 = win.cpu_reader.borrow_mut().get_addressing_mode_list();

        win.cpu_reader.borrow_mut().add_new_label("reset".to_string(),0x0, 1);

        refresh();//needed for win size
        win.menu1 = Windows::create_menu(&mut win.items1,win.win1,0);
        win.menu2 = Windows::create_menu(&mut win.items2,win.win3,0);
        win.menu3 = Windows::create_menu(&mut win.items3,win.win4,0);
        
        win.screen_height = 0;
        win.screen_width = 0;
        win.resize_check();

        win
    }

    pub fn destroy(&mut self) {
        Windows::destroy_menu(self.menu1,&mut self.items1);
        Windows::destroy_menu(self.menu2,&mut self.items2);
        Windows::destroy_menu(self.menu3,&mut self.items3);
        Windows::destroy_win(self.win2_sub);
        Windows::destroy_win(self.win1);
        Windows::destroy_win(self.win2);
        Windows::destroy_win(self.win3);
        Windows::destroy_win(self.win4);
        Windows::destroy_win(self.win5);
        clear();
        endwin();
    }

    pub fn run_virpc(&mut self) {
        self.virpc.run();
    }

    //////////////////////////////////////////////
    // Window draw functions
    //////////////////////////////////////////////

    //window dimensions
    fn wd (&mut self, window : u32, dimension : char) -> i32 {
        let sh = self.screen_height;
        let sw = self.screen_width;
        static WIN1_MAXWIDTH : i32 = 35;
        static WIN4_HEIGHT : i32 = 4;
        //static WIN12_HEIGHT : u32 = 25;
        //static WIN3_MAXWIDTH : u32 = 30;

        match window {
            1 => {
                match dimension {
                    'y' => { 1 },
                    'x' => { 0 },
                    'h' => { (sh/2)-1 },
                    'w' => { WIN1_MAXWIDTH },
                    'u' => { sh/2 },//y-end
                    'v' => { WIN1_MAXWIDTH },//x-end
                    _ => { 0 },
                }
            }
            2 => {
                match dimension {
                    'y' => { 1 }, 
                    'x' => { WIN1_MAXWIDTH },
                    'h' => { (sh/2)-1 },
                    'w' => { sw-WIN1_MAXWIDTH },
                    'u' => { sh/2 },
                    'v' => { sw },
                    _ => { 0 },
                }
            }
            3 => {
                match dimension {
                    'y' => { sh/2 },
                    'x' => { 0 },
                    'h' => { sh/2-WIN4_HEIGHT },
                    'w' => { WIN1_MAXWIDTH },
                    'u' => { sh-WIN4_HEIGHT },
                    'v' => { WIN1_MAXWIDTH },
                    _ => { 0 },
                }
            }
            4 => {
                match dimension {
                    'y' => { sh-WIN4_HEIGHT },
                    'x' => { 0 },
                    'h' => { WIN4_HEIGHT },
                    'w' => { WIN1_MAXWIDTH },
                    'u' => { sh },
                    'v' => { WIN1_MAXWIDTH },
                    _ => { 0 },
                }
            }
            5 => {
                match dimension {
                    'y' => { sh/2 }, 
                    'x' => { WIN1_MAXWIDTH },
                    'h' => { sh/2 },
                    'w' => { sw-WIN1_MAXWIDTH },
                    'u' => { sh },
                    'v' => { sw },
                    _ => { 0 },
                }
            }
            _ => { 0 },
        }

    }

    //call to check if we need to redraw the windows due to a resize
    pub fn resize_check(&mut self) {
        getmaxyx(stdscr(), &mut self.screen_height_n, &mut self.screen_width_n);
        if self.screen_height != self.screen_height_n || self.screen_width != self.screen_width_n {
            self.screen_height = self.screen_height_n;
            self.screen_width = self.screen_width_n;
            let ch = ' ' as chtype;

            wborder(self.win1, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win1);
            mvwin(self.win1, self.wd(1,'y'), self.wd(1,'x'));
            wresize(self.win1,self.wd(1,'h'), self.wd(1,'w'));
            box_(self.win1,0,0);
            
            wborder(self.win2, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win2);
            mvwin(self.win2, self.wd(2,'y'), self.wd(2,'x'));
            wresize(self.win2,self.wd(2,'h'), self.wd(2,'w'));
            box_(self.win2,0,0);
            
            wborder(self.win3, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win3);
            mvwin(self.win3, self.wd(3,'y'), self.wd(3,'x'));
            wresize(self.win3,self.wd(3,'h'), self.wd(3,'w'));
            box_(self.win3,0,0);
            
            wborder(self.win4, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win4);
            mvwin(self.win4, self.wd(4,'y'), self.wd(4,'x'));
            wresize(self.win4,self.wd(4,'h'), self.wd(4,'w'));
            box_(self.win4,0,0);

            wborder(self.win5, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win5);
            mvwin(self.win5, self.wd(5,'y'), self.wd(5,'x'));
            wresize(self.win5,self.wd(5,'h'), self.wd(5,'w'));
            box_(self.win5,0,0);

            self.refresh_screen();
        }
    }

    pub fn refresh_fast(&mut self) {
        self.current_pc = self.cpu_reader.borrow_mut().read_int_le(0xF000);
        let status = match self.virpc.status() {
            true => "running",
            false => "stopped",
        };
        let s = format!("edit:{:08X},current:{:08X} {} <F5 run/pause> <F6 reset> <F8 step> <F9 breakpoint>",self.edit_line,self.current_pc, status);
        mvprintw(0,0,s.as_str());
        refresh();   
    }

    fn refresh_screen(&mut self) {
        match self.focus {
            0 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1,"<code>");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {} ",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
                mvwprintw(self.win5,0,1," memory view ");
            }
            1 => {
                mvwprintw(self.win1,0,1,"<commands>");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {} ",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
                mvwprintw(self.win5,0,1," memory view ");
            }
            2 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!("<variables> - arg {} ",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
                mvwprintw(self.win5,0,1," memory view ");
            }
            3 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {} ",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1,"<addressing mode>");
                mvwprintw(self.win5,0,1," memory view ");
            }
            4 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {} ",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
                mvwprintw(self.win5,0,1,"<memory view>");
            }
            _ => { 
            }
        }
        refresh();            
        wrefresh(self.win1);
        wrefresh(self.win2);
        wrefresh(self.win3);
        wrefresh(self.win4);

        self.refresh_code();

        Windows::destroy_menu(self.menu1, &mut self.items1);
        self.items1 = self.cpu_reader.borrow_mut().get_commands_list();
        self.menu1 = Windows::create_menu(&mut self.items1, self.win1, self.menu1_choice);

        Windows::destroy_menu(self.menu2, &mut self.items2);
        self.items2 = self.cpu_reader.borrow_mut().get_data_list();
        self.menu2 = Windows::create_menu(&mut self.items2 ,self.win3, self.menu2_choice);

        Windows::destroy_menu(self.menu3, &mut self.items3);
        self.items3 = self.cpu_reader.borrow_mut().get_addressing_mode_list();
        self.menu3 = Windows::create_menu(&mut self.items3, self.win4, self.menu3_choice);

        let adr = self.cpu_reader.borrow_mut().get_data_value(self.menu2_choice);

        let size = match self.cpu_reader.borrow_mut().get_label(adr) {
            Some(lbl) => { lbl.size }
            None => { 1 }//assume size of 1
        };
        self.set_memview_focus(adr,size);
        self.refresh_memview();
    }

    fn refresh_code(&mut self) {
        let mut lpc = 0;
        let mut tpc;

        let mut end = (self.screen_height/2-3) as u32;
        if (self.edit_line + ((self.wd(2,'h')-2)/2) as u32) > (self.wd(2,'h')-2) as u32 {
            end = self.edit_line + ((self.wd(2,'h')-2)/2) as u32;
        }
        
        wattrset(self.win2_sub, COLOR_PAIR(1));
        wresize(self.win2_sub,self.wd(2,'h')-2, self.wd(2,'w')-2);
        wmove(self.win2_sub,0,0);

        self.cpu_reader.borrow_mut().data.clear();
        self.cpu_reader.borrow_mut().data = cpu::CPU::get_variables_list();

        for i in 0..end {
            tpc = self.cpu_reader.borrow_mut().disassemble(lpc);
            if i == self.edit_line {
                wattrset(self.win2_sub, COLOR_PAIR(2));

                self.edit_pc = lpc;
                
                if self.edit_cmd == -1 {//menu1
                    self.menu1_choice = self.cpu_reader.borrow_mut().get_instruction_index();
                }
                else {
                        self.menu1_choice = self.edit_cmd as u32;
                }
                
                if self.edit_item[self.cur_arg as usize] == -1 {//menu2
                    self.menu2_choice = self.cpu_reader.borrow_mut().instruction.arg_index[self.cur_arg as usize];
                }
                else {
                    self.menu2_choice = self.edit_item[self.cur_arg as usize] as u32;
                }
                
                if self.edit_mode[self.cur_arg as usize] == -1 {//menu3
                    self.menu3_choice = self.cpu_reader.borrow_mut().argument_type(self.cur_arg);
                }
                else {
                    self.menu3_choice = self.edit_mode[self.cur_arg as usize] as u32;
                }
            }
            if i >= end - ((self.wd(2,'h')-2) as u32) {
                if self.current_pc >= lpc && self.current_pc < tpc && self.virpc.status() == false {
                    wattrset(self.win2_sub, COLOR_PAIR(3));
                }
                wprintw(self.win2_sub, self.cpu_reader.borrow_mut().instruction_to_text().as_str());
                wattrset(self.win2_sub, COLOR_PAIR(1));
            }
            lpc = tpc;
        }
        //set currenl line to the one we want to edit
        self.cpu_reader.borrow_mut().load_opcode_data(self.edit_pc);//fill data with current instruction
        self.cpu_reader.borrow_mut().set_pc(self.edit_pc);//set pc to current instruction
        //refresh the screen
        wrefresh(self.win2_sub);
    }

    fn refresh_memview(&mut self) {
        let w = (self.wd(5,'w')/4)-4;
        for i in 0..(self.wd(5,'h')-2) {
            if ( self.hex_view_offset + (i*w) as u32 )  >= MEMORY_SIZE { break; }

            let s = format!("${:08X} |",self.hex_view_offset + (i*w) as u32);
            mvwprintw(self.win5,i+1,1,s.as_str());
            for j in 0..w {
                let adr = self.hex_view_offset + ( (i*w) + j ) as u32;
                         
                if adr >= self.mem_highlight && adr < self.mem_highlight + self.mem_highlight_size {
                    if adr > self.mem_highlight {
                        wattrset(self.win5, COLOR_PAIR(2));
                        wprintw(self.win5," ");
                    }
                    else {
                        wprintw(self.win5," ");
                        wattrset(self.win5, COLOR_PAIR(2));
                    }
                }
                else { wprintw(self.win5," "); }

                let val : u8 = self.cpu_reader.borrow_mut().read_byte(adr);
                if adr < MEMORY_SIZE {
                    wprintw(self.win5,format!("{:02X}",val).as_str());
                }
                else {
                    wprintw(self.win5,"  ");
                }
                wattrset(self.win5, COLOR_PAIR(1));     
            }
            wprintw(self.win5," | ");
            for j in 0..w {
                let adr = self.hex_view_offset + ( (i*w) + j ) as u32;
                let val : u8 = self.cpu_reader.borrow_mut().read_byte(adr);
                if adr < MEMORY_SIZE {
                    wprintw(self.win5,format!("{}",EBCDIC[val as usize]).as_str());
                }
                else {
                    wprintw(self.win5," ");
                }
            }
        }
        wrefresh(self.win5);
    }

    fn set_memview_focus(&mut self, adr : u32, size : u32) {
        self.mem_address = adr;
        self.hex_view_offset = self.mem_address - (self.mem_address % ((self.wd(5,'w')/4)-4) as u32);//scroll so mem_address is in view
        self.mem_highlight = self.mem_address; 
        self.mem_highlight_size = size;
    }

    ////////////////////////////////////////////
    // handling of new value
    ////////////////////////////////////////////

    fn new_val(&mut self) {
        //and opcode: edit_cmd or (self.cpu_reader.borrow_mut().instruction_u8)
        let mut code : u8;
        if self.edit_cmd != -1 { code = self.edit_cmd as u8; }
        else { code = self.cpu_reader.borrow_mut().instruction_u8 >> 4; }
        code &= 0x0f;

        match self.edit_item[self.cur_arg as usize] {
            0 => { self.input_register(); }
            1 if self.cur_arg == 1 && code == 0x00 => { self.input_jmp_opts(); }
            //edit cpu-arg and opcode if necesary: self.edit_cmd, self.edit_mode[0], self.edit_mode[1], self.edit_mode[2]
            /* 2 if self.cur_arg == 1 && code == 0x0a  => {//ldr
            }
            2 if self.cur_arg == 2 && code == 0x0a=> {//ldr
            }
            2 if self.cur_arg == 1 && code == 0x0b=> {//str
            }
            2 if self.cur_arg == 2 && code == 0x0b=> {//str
            }*/            
            2 if self.cur_arg == 1 && code == 0x01 => {//call or jmp
                self.input_code_label(); //refresh_screen code for help              
            }
            2 => { 
                self.input_mem_label(); 
            }//mem
            3 => { 
                self.input_bss(); 
            }
            _ => {
                self.input_value();
            },
        }
        self.edit_item[self.cur_arg as usize] = -1;
    }

    fn input_jmp_opts(&mut self) {
        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));

        let s;
        let v;
        let d = " ".to_string();

        mvwprintw(lwin_menu,0,1," select  jump options ");
        let mut items = self.cpu_reader.borrow_mut().jmp_opts();
        let mut select :i32 = 0;
        let menu = Windows::create_menu(&mut items,lwin_menu,select as u32);
        //provide menu with all jmp options
        //use self.edit_item[self.cur_arg as usize] to determine the menu-option
        wrefresh(lwin_menu);
        let mut ch = getch();
        while ch != 27 as i32 { // ESC pressed, so quit
            match ch {
                KEY_UP => {
                    menu_driver(menu, REQ_UP_ITEM);
                    wrefresh(lwin_menu);
                }
                KEY_DOWN => {
                    menu_driver(menu, REQ_DOWN_ITEM);
                    wrefresh(lwin_menu);
                }
                0xa => {
                    select = item_index(current_item(menu));
                    s = format!("CONST_{:08X}",select);
                    v = select as u32;
                    self.edit_mode[1] = 0;
                    self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_item(s, d, v) ) as i32;
                    self.modify();
                    break;
                }
                _ => {
                    menu_driver(menu, ch);
                    wrefresh(lwin_menu);
                }
            }
            ch = getch();
        }
        Windows::destroy_menu(menu,&mut items);
        Windows::destroy_win(lwin_menu);
    }

    fn input_register(&mut self) {
        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));

        let s;
        let v;
        let d = " ".to_string();

        mvwprintw(lwin_menu,0,1," select register ");
        let mut items = self.cpu_reader.borrow_mut().reg_opts();
        let mut select :i32 = 0;
        let menu = Windows::create_menu(&mut items,lwin_menu,select as u32);
        wrefresh(lwin_menu);
        let mut ch = getch();
        while ch != 27 as i32 { // ESC pressed, so quit
            match ch {
                KEY_UP => {
                    menu_driver(menu, REQ_UP_ITEM);
                    wrefresh(lwin_menu);
                }
                KEY_DOWN => {
                    menu_driver(menu, REQ_DOWN_ITEM);
                    wrefresh(lwin_menu);
                }
                0xa => {
                    select = item_index(current_item(menu));
                    s = format!("REG_{}",select);
                    v = ((select * 4)+0xF000) as u32;
                    //write direct (cur_arg = 0) or indirect (cur_arg = 1)
                    if self.cur_arg == 0 { self.edit_mode[0] = 0; }
                    else { self.edit_mode[self.cur_arg as usize] = 1; }

                    self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_item(s, d, v) ) as i32;
                    self.modify();
                    break;
                }
                _ => {
                    menu_driver(menu, ch);
                    wrefresh(lwin_menu);
                }
            }
            ch = getch();
        }
        Windows::destroy_menu(menu,&mut items);
        Windows::destroy_win(lwin_menu);
    }

    fn input_value(&mut self) {
        //menu for a new value based on argument-index (self.cur_arg) 
        //new window, asking to input a value
        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));

        let s;
        let d = " ".to_string();
        mvwprintw(lwin_menu,0,1," input a number ");
        mvwprintw(lwin_menu,2,1," input:");
        wrefresh(lwin_menu);
        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        wrefresh(lwin_menu);
        let mut val : String = String::from("");
        let mut ch = 0;//getch();
        while ch != 27 as i32 { // ESC pressed, so quit
            //hex or normal
            ch = getch();
            match ch {
                //KEY_LEFT => {}
                //KEY_RIGHT => {}
                0xa => {//enter
                    let v = Windows::string_to_val(val);
                    s = format!("CONST_{:08X}",v);
                    self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_item(s, d, v) ) as i32;
                    self.modify();
                    break;
                } 
                0x107 => {//backspace
                    val.pop();
                }
                _ => {
                    let key = (ch as u8) as char;
                    if ( key.is_ascii_hexdigit() || ch==0x78 || ch==0x58 ) && val.len() < 10 {
                        val.push(key);
                    }
                }
            }
            mvwprintw(lwin_menu,2,8,"          ");
            mvwprintw(lwin_menu,2,8,val.as_str());
            wrefresh(lwin_menu);
        }
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        Windows::destroy_win(lwin_menu);
    }

    fn input_bss(&mut self) {
        //dialog for BSS: select an address (provide suggestions based on labels in range bss)
        // refresh_screen memview for help
        let mut val : Vec<String> = Vec::new();//String::from("");
        let adr = self.cpu_reader.borrow_mut().get_free_bss();
        val.push(format!("0x{:08X}",adr).to_string());
        val.push(format!("BSS_{:08X}",adr).to_string());
        val.push("1".to_string());

        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));
        mvwprintw(lwin_menu,0,1," add a bss item");
        mvwprintw(lwin_menu,3,1,format!(" name:\t\t{}",val[1]).as_str());
        mvwprintw(lwin_menu,4,1,format!(" size:\t\t{}",val[2]).as_str());
        mvwprintw(lwin_menu,2,1,format!(" address:\t{}",val[0]).as_str());

        let mut ll = self.cpu_reader.borrow_mut().get_mem_label_list();
        self.handle_input_menu(lwin_menu,&mut val, &mut ll);
    }

    fn input_code_label(&mut self) {
        //dialog for BSS: select an address (provide suggestions based on labels in range bss)
        // refresh_screen memview for help
        let mut val : Vec<String> = Vec::new();//String::from("");
        let adr = self.edit_pc;
        val.push(format!("0x{:08X}",adr).to_string());
        val.push(format!("LBL_{:08X}",adr).to_string());
        val.push("1".to_string());

        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));
        mvwprintw(lwin_menu,0,1," add a code label");
        mvwprintw(lwin_menu,3,1,format!(" name:\t\t{}",val[1]).as_str());
        mvwprintw(lwin_menu,4,1,format!(" size:\t\t{}",val[2]).as_str());
        mvwprintw(lwin_menu,2,1,format!(" address:\t{}",val[0]).as_str());

        let mut ll = self.cpu_reader.borrow_mut().get_code_label_list();
        self.handle_input_menu(lwin_menu,&mut val, &mut ll);
    }

    fn input_mem_label(&mut self) {
        //dialog for new heap: select an address (provide suggestions, based on labels in range mem)
        // refresh_screen memview for help

        let mut val : Vec<String> = Vec::new();//String::from("");
        let adr = self.cpu_reader.borrow_mut().get_free_mem();
        val.push(format!("0x{:08X}",adr).to_string());
        val.push(format!("VAR_{:08X}",adr).to_string());
        val.push("1".to_string());

        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);

        let lwin_menu = Windows::create_win(" ",self.wd(3,'h'), self.wd(3,'w'), self.wd(3,'y'), self.wd(3,'x'));
        mvwprintw(lwin_menu,0,1," add a label <MEM>");
        mvwprintw(lwin_menu,3,1,format!(" name:\t\t{}",val[1]).as_str());
        mvwprintw(lwin_menu,4,1,format!(" size:\t\t{}",val[2]).as_str());
        mvwprintw(lwin_menu,2,1,format!(" address:\t{}",val[0]).as_str());

        let mut ll = self.cpu_reader.borrow_mut().get_mem_label_list();
        self.handle_input_menu(lwin_menu,&mut val, &mut ll);
    }

    fn handle_input_menu(&mut self, lwin_menu : WINDOW, val :&mut Vec<String>, list : &mut Vec<ITEM>) {
        let mut curval = 0;
        let d = "".to_string();

        let subwin = derwin(lwin_menu,self.wd(3,'h')-(3 + val.len() as i32),self.wd(3,'w')-2, 2 + val.len() as i32, 1);
        let menu = Windows::create_menu(list,subwin,100);
        unpost_menu(menu);
        menu_opts_off(menu, O_SHOWDESC);
        set_menu_fore(menu, COLOR_PAIR(COLOR_PAIR_DEFAULT));
        post_menu(menu);
        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        mvwprintw(lwin_menu,2+curval as i32,16,"            ");
        mvwprintw(lwin_menu,2+curval as i32,16,val[curval].as_str());
        wrefresh(lwin_menu);

        let mut existing = false;
        let mut ch = 0;
        while ch != 27 as i32 { // ESC pressed, so quit
            //hex or normal
            ch = getch();
            match ch {
                KEY_UP => { 
                    if existing {
                        menu_driver(menu, REQ_UP_ITEM);
                        let s = item_description(current_item(menu)).clone();
                        let lval : u32 = s.parse().unwrap();
                        let lbl =  match self.cpu_reader.borrow_mut().get_label(lval) {
                            Some(ll) => { ll },
                            None => {cpu::Label { tag : "UNKNOOWN".to_string(), address : 0x0, size : 0 }},
                        };
                        val[0] = format!("{:08X}",lbl.address);
                        val[1] = lbl.tag.clone();
                        val[2] = lbl.size.to_string();
                        mvwprintw(lwin_menu,2,16,format!("{:14}",val[0]).as_str());
                        mvwprintw(lwin_menu,3,16,format!("{:14}",val[1]).as_str());
                        mvwprintw(lwin_menu,4,16,format!("{:14}",val[2]).as_str());
                        self.set_memview_focus(lbl.address,lbl.size);
                        self.refresh_memview();
                    }
                    else {
                        if curval > 0 {
                            curval = curval - 1;
                        }  
                    }
                }
                KEY_DOWN => { 
                    if existing {
                        menu_driver(menu, REQ_DOWN_ITEM);
                        let s = item_description(current_item(menu)).clone();
                        let lval : u32 = s.parse().unwrap();
                        let lbl =  match self.cpu_reader.borrow_mut().get_label(lval) {
                            Some(ll) => { ll },
                            None => {cpu::Label { tag : "UNKNOOWN".to_string(), address : 0x0, size : 0 }},
                        };
                        val[0] = format!("{:08X}",lbl.address);
                        val[1] = lbl.tag.clone();
                        val[2] = lbl.size.to_string();
                        mvwprintw(lwin_menu,2,16,format!("{:14}",val[0]).as_str());
                        mvwprintw(lwin_menu,3,16,format!("{:14}",val[1]).as_str());
                        mvwprintw(lwin_menu,4,16,format!("{:14}",val[2]).as_str());
                        self.set_memview_focus(lbl.address,lbl.size);
                        self.refresh_memview();
                    }
                    else {
                        curval = (curval + 1) % 3; 
                    }
                }
                0x09 => { //tab
                    //curval = (curval + 1) % 3; 
                    if existing {
                        existing = false;
                        //unpost_menu(menu);
                        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
                        unpost_menu(menu);
                        menu_opts_off(menu, O_SHOWDESC);
                        set_menu_fore(menu, COLOR_PAIR(COLOR_PAIR_DEFAULT));
                        post_menu(menu);
                    }
                    else {
                        existing = true;
                        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
                        let s = item_description(current_item(menu)).clone();
                        let lval : u32 = s.parse().unwrap();
                        let lbl =  match self.cpu_reader.borrow_mut().get_label(lval) {
                            Some(ll) => { ll },
                            None => {cpu::Label { tag : "UNKNOOWN".to_string(), address : 0x0, size : 0 }},
                        };
                        val[0] = format!("{:08X}",lbl.address);
                        val[1] = lbl.tag.clone();
                        val[2] = lbl.size.to_string();
                        mvwprintw(lwin_menu,2,16,format!("{:14}",val[0]).as_str());
                        mvwprintw(lwin_menu,3,16,format!("{:14}",val[1]).as_str());
                        mvwprintw(lwin_menu,4,16,format!("{:14}",val[2]).as_str());
                        self.set_memview_focus(lbl.address,lbl.size);
                        self.refresh_memview();
                        unpost_menu(menu);
                        menu_opts_off(menu, O_SHOWDESC);
                        set_menu_fore(menu, COLOR_PAIR(COLOR_PAIR_KEYWORD));
                        post_menu(menu);
                    }
                }
                0xa => {//enter
                    if existing {
                        let s = item_description(current_item(menu)).clone();
                        let lval : u32 = s.parse().unwrap();
                        let mut ss = "".to_string();
                        match self.cpu_reader.borrow_mut().get_label(lval) {
                            Some(ll) => { ss = ll.tag.clone(); },
                            None => {},
                        }
                        self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_item(ss, d, lval)) as i32;
                    }
                    else {
                        let v = Windows::string_to_val(val[0].clone());
                        self.cpu_reader.borrow_mut().add_new_label(val[1].clone(),v,Windows::string_to_val(val[2].clone()));
                        self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_item(val[1].clone(), d, v)) as i32;
                    }
                    self.modify();
                    break;
                } 
                0x107 if existing == false => {//backspace
                    if existing {

                    }
                    else {
                        val[curval].pop();
                    }
                }
                _ => {
                    if existing {
                        menu_driver(menu, ch);
                    }
                    else {
                        let key = (ch as u8) as char;
                        if curval == 0 || curval == 2 {
                            if ( key.is_ascii_hexdigit() || ch==0x78 || ch==0x58 ) && val[curval].len() < 12 {
                                val[curval].push(key);
                            }
                        }
                        else {
                            if val[curval].len() < 12 {
                                val[curval].push(key);
                            }
                        }
                    }
                }
            }
            wrefresh(subwin);
            //mvwprintw(lwin_menu,2+curval as i32,16,"            ");
            mvwprintw(lwin_menu,2+curval as i32,16,format!("{:14}",val[curval]).as_str());
            wmove(lwin_menu,2+curval as i32,16+val[curval].len() as i32);
            wrefresh(lwin_menu);
        }
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        Windows::destroy_menu(menu,list);
        Windows::destroy_win(subwin);
        Windows::destroy_win(lwin_menu);
    }

    //find labels in code or mem
    fn search_label(&mut self) {
        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",self.wd(1,'h') + self.wd(3,'h') + self.wd(4,'h'), self.wd(3,'w'), self.wd(1,'y'), self.wd(1,'x'));

        let mut items = Vec::new();
        if self.focus == 0 {
            mvwprintw(lwin_menu,0,1," search label <CODE>");
            items = self.cpu_reader.borrow_mut().get_code_label_list();
        }
        if self.focus == 4 {
            mvwprintw(lwin_menu,0,1," search label <MEM>");
            items = self.cpu_reader.borrow_mut().get_mem_label_list();
        }
        let menu = Windows::create_menu(&mut items,lwin_menu,0);
        wrefresh(lwin_menu);

        let mut ch = 0;
        while ch != 27 as i32 { // ESC pressed, so quit
            ch = getch();
            match ch {
                KEY_UP => { 
                    menu_driver(menu, REQ_UP_ITEM);
                    let s = item_description(current_item(menu)).clone();
                    let lval : u32 = s.parse().unwrap();
                    let lbl =  match self.cpu_reader.borrow_mut().get_label(lval) {
                        Some(ll) => { ll },
                        None => {cpu::Label { tag : "UNKNOOWN".to_string(), address : 0x0, size : 0 }},
                    };
                    self.set_memview_focus(lbl.address,lbl.size);
                    self.refresh_memview();
                    if self.focus == 0 {
                        self.reset_edit();
                        let mut lpc = lbl.address;
                        let tmp = self.edit_line;
                        self.edit_line = 0;
                        let mut tpc;

                        wattrset(self.win2_sub, COLOR_PAIR(1));
                        wresize(self.win2_sub,self.wd(2,'h')-2, self.wd(2,'w')-2);
                        let end = (self.wd(2,'h')-2) as u32;
                        wmove(self.win2_sub,0,0);
                        for i in 0..end {
                            tpc = self.cpu_reader.borrow_mut().disassemble(lpc);
                            if i == self.edit_line {
                                wattrset(self.win2_sub, COLOR_PAIR(2));
                                self.edit_pc = lpc;
                            }
                            wprintw(self.win2_sub, self.cpu_reader.borrow_mut().instruction_to_text().as_str());
                            wattrset(self.win2_sub, COLOR_PAIR(1));
                            lpc = tpc;
                        }
                        self.cpu_reader.borrow_mut().set_pc(self.edit_pc);//set pc to current instruction
                        wrefresh(self.win2_sub);
                        self.edit_line = tmp;
                    }
                }
                KEY_DOWN => { 
                    menu_driver(menu, REQ_DOWN_ITEM);
                    let s = item_description(current_item(menu)).clone();
                    let lval : u32 = s.parse().unwrap();
                    let lbl =  match self.cpu_reader.borrow_mut().get_label(lval) {
                        Some(ll) => { ll },
                        None => {cpu::Label { tag : "UNKNOOWN".to_string(), address : 0x0, size : 0 }},
                    };
                    self.set_memview_focus(lbl.address,lbl.size);
                    self.refresh_memview();
                    if self.focus == 0 {
                        self.reset_edit();
                        let mut lpc = lbl.address;
                        let tmp = self.edit_line;
                        self.edit_line = 0;
                        let mut tpc;

                        wattrset(self.win2_sub, COLOR_PAIR(1));
                        wresize(self.win2_sub,self.wd(2,'h')-2, self.wd(2,'w')-2);
                        let end = (self.wd(2,'h')-2) as u32;
                        wmove(self.win2_sub,0,0);
                        for i in 0..end {
                            tpc = self.cpu_reader.borrow_mut().disassemble(lpc);
                            if i == self.edit_line {
                                wattrset(self.win2_sub, COLOR_PAIR(2));
                                self.edit_pc = lpc;
                            }
                            wprintw(self.win2_sub, self.cpu_reader.borrow_mut().instruction_to_text().as_str());
                            wattrset(self.win2_sub, COLOR_PAIR(1));
                            lpc = tpc;
                        }
                        self.cpu_reader.borrow_mut().set_pc(self.edit_pc);//set pc to current instruction
                        wrefresh(self.win2_sub);
                        self.edit_line = tmp;
                    }
                }
                0xa => {
                    //TODO if pressed enter, scroll to currently selected result
                }
                _ => {
                    menu_driver(menu, ch);
                }
            }
            wrefresh(lwin_menu);
        }

        Windows::destroy_menu(menu,&mut items);
        Windows::destroy_win(lwin_menu);
        self.screen_height = 0;//trigger an refresh_screen
        self.resize_check();//show the edited value
    }

    //modify values based on sub-meny
    fn modify(&mut self) {
        //take all current settings from menu 1, 2 and 3
        self.cpu_reader.borrow_mut().set_opcode(self.edit_cmd, self.edit_mode[0], self.edit_mode[1], self.edit_mode[2]);
        //from arg, take 1, 2 and 3 based on size
        self.cpu_reader.borrow_mut().parse_args(self.edit_item[0],self.edit_item[1],self.edit_item[2]);
        //write bytecode
        self.cpu_reader.borrow_mut().assemble();
    }

    //reset modify values
    fn reset_edit(&mut self) {
        self.edit_cmd = -1;
        self.edit_item[0] = -1;
        self.edit_item[1] = -1;
        self.edit_item[2] = -1;
        self.edit_mode[0] = -1;
        self.edit_mode[1] = -1;
        self.edit_mode[2] = -1;
    }
    ///////////////////////////////////////////
    // key event handler
    ///////////////////////////////////////////
    pub fn handle_keys(&mut self, ch : i32) {
        match ch {
            0x09 => {
                self.focus += 1;
                if self.focus > 4 {
                    self.focus = 0;
                }
                self.refresh_screen();
            }
            0xa => {
                if self.edit_item[self.cur_arg as usize] > -1 && self.edit_item[self.cur_arg as usize] < 4 {
                    self.new_val();
                }
                else {
                    self.modify();//edit the current value
                }
                self.screen_height = 0;//trigger an refresh_screen
                self.resize_check();//show the edited value
            }
            0x10d => {//<F5 run/pause>
                //toggle running/stop
                if self.run_program == false {
                    self.virpc.continue_cpu();
                    self.run_program = true;
                }
                else {
                    self.virpc.stop();
                    self.run_program = false;
                }
                self.refresh_fast();
                self.refresh_code();
            }
            0x10e => {//<F6 reset>
                //set pc of virpc to 0
                self.virpc.reset();
                self.refresh_fast();
                self.refresh_code();
            }
            0x110 => {//<F8 step>
                //perform a cpu-step
                self.virpc.continue_cpu();
                self.virpc.run();
                self.virpc.stop();
                self.refresh_fast();
                self.refresh_code();
            }
            0x111 => {//<F9 breakpoint>
                //set virpc::toggle-breakpoint
                self.virpc.breakpoint(self.edit_pc);
            }
            _ => {
                match self.focus {
                    0 => self.handle_keys_win2(ch),
                    1 => self.handle_keys_win1(ch),
                    2 => self.handle_keys_win3(ch),
                    3 => self.handle_keys_win4(ch),
                    4 => self.handle_keys_win5(ch),
                    _ => {},
                };
            }
        }
    }

    fn handle_keys_win1(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                menu_driver(self.menu1, REQ_UP_ITEM);
                wrefresh(self.win1);
                self.edit_cmd = item_index(current_item(self.menu1));
            }
            KEY_DOWN => {
                menu_driver(self.menu1, REQ_DOWN_ITEM);
                wrefresh(self.win1);
                self.edit_cmd = item_index(current_item(self.menu1));
            }
            _ => {
                menu_driver(self.menu1, ch);
                wrefresh(self.win1);
                self.edit_cmd = item_index(current_item(self.menu1));
            }
        }
    }

    fn handle_keys_win2(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                if self.edit_line > 0 {
                   self.edit_line -= 1; 
                }
                self.reset_edit();
                self.refresh_screen();
            }
            KEY_DOWN => {
                self.edit_line += 1;
                self.reset_edit();
                self.refresh_screen();
            }
            0x66 => {
                self.search_label();
            }
            _ => {
            }
        }
    }

    fn handle_keys_win3(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                menu_driver(self.menu2, REQ_UP_ITEM);
                wrefresh(self.win3);
                self.edit_item[self.cur_arg as usize] = item_index(current_item(self.menu2));
            }
            KEY_DOWN => {
                menu_driver(self.menu2, REQ_DOWN_ITEM);
                wrefresh(self.win3);
                self.edit_item[self.cur_arg as usize] = item_index(current_item(self.menu2));
            }
            KEY_LEFT => {
                if self.cur_arg > 0 {
                    self.cur_arg -= 1;
                    self.refresh_screen();
                }
            }
            KEY_RIGHT => {
                if self.cur_arg < 2 {
                    self.cur_arg += 1;
                    self.refresh_screen();                    
                }
            }
            _ => {
                menu_driver(self.menu2, ch);
                wrefresh(self.win3);
                self.edit_item[self.cur_arg as usize] = item_index(current_item(self.menu2));
            }
        }
    }

    fn handle_keys_win4(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                menu_driver(self.menu3, REQ_UP_ITEM);
                wrefresh(self.win4);
                self.edit_mode[self.cur_arg as usize] = item_index(current_item(self.menu3));
            }
            KEY_DOWN => {
                menu_driver(self.menu3, REQ_DOWN_ITEM);
                wrefresh(self.win4);
                self.edit_mode[self.cur_arg as usize] = item_index(current_item(self.menu3));
            }
            _ => {
                menu_driver(self.menu3, ch);
                wrefresh(self.win4);
                self.edit_mode[self.cur_arg as usize] = item_index(current_item(self.menu3));
            }
        }
    }

    fn handle_keys_win5(&mut self, ch : i32) {
        let w = ((self.wd(5,'w')/4)-4) as u32;
        match ch {
            KEY_UP => {
                if self.hex_view_offset >= w {
                    self.hex_view_offset -= w;
                }
                else {
                    self.hex_view_offset = 0;
                }
                self.refresh_memview();
            }
            KEY_DOWN => {
                if self.hex_view_offset < MEMORY_SIZE-1 {
                    self.hex_view_offset += w;
                }
                self.refresh_memview();
            }            
            0x65 => {//e pressed, means edit hex values, until esc pressed
                self.refresh_memview();
                curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
                
                let mut hex = true;
                let mut begin : i32 = 13;
                let mut col = begin;
                let mut row = 1;
                let mut w = ((self.wd(5,'w')/4)*3)-1;
                let h = self.wd(5,'h') - 2;

                let mut ch = 0;
                //TODO: esc cannot be caught due to catching on higher level
                while ch != 27 as i32 { // ESC pressed, so quit
                    self.refresh_memview();
                    wmove(self.win5, row, col);
                    wrefresh(self.win5);
                    ch = getch();
                    match ch {
                        KEY_LEFT => {
                            if col > begin { col -= 1; }
                            else {
                                if row > 1 { col = w; row -= 1; }  
                                else {
                                    let mut r = w-begin;
                                    if hex { r /= 3; }
                                    if self.hex_view_offset as i32 - r > 0 { 
                                        self.hex_view_offset -= (r+1) as u32; 
                                        col = w;
                                    }
                                    else { 
                                        self.hex_view_offset = 0; 
                                        col = begin;
                                    }
                                }
                            }
                            if (col) % 3 == 0 && hex { col -= 1; }
                        }
                        KEY_RIGHT => {
                            if col < w { col += 1; }
                            else {
                                if row < h { col = begin; row += 1; }
                                else { 
                                    col = begin;
                                    let mut r = w-begin;
                                    if hex { r /= 3; }
                                    self.hex_view_offset += (r+1) as u32; 
                                }
                            }

                            if (col) % 3 == 0 && hex { col += 1; }
                        }
                        KEY_UP => {
                            let mut r = w-begin;
                            if hex { r /= 3; }
                            if row > 1 { row -= 1; }  
                            else {
                                if self.hex_view_offset as i32 - (r+1) >= 0 { 
                                    self.hex_view_offset -= (r+1) as u32; 
                                }
                                else { self.hex_view_offset = 0; }
                            }
                        }
                        KEY_DOWN => {
                            if row < h { row += 1; }
                            else { 
                                let mut r = w-begin;
                                if hex { r /= 3; }
                                self.hex_view_offset += (r+1) as u32; 
                            }
                        }
                        0x9 => {//tab switch from hex to mem and back
                            if hex {
                                hex = false;
                                col -= begin;//remove offset
                                begin = ((self.wd(5,'w')/4)*3)+3;//redefine begin
                                w = ((self.wd(5,'w')/4)*4)-2;
                                col = begin + (col/3);
                            }
                            else {
                                hex = true;
                                col -= begin;//remove offset
                                begin = 13;//redefine begin
                                w = ((self.wd(5,'w')/4)*3)-1;
                                col = begin + (col*3);
                            }
                        }
                        _ => {
                            let key = (ch as u8) as char;
                            if (key.is_ascii() && !hex) || (hex && key.is_ascii_hexdigit()) {
                                if hex {			// if in hex win...   
                                    let addr = self.hex_view_offset + (((col-begin)/3) + ((row-1)*(((w+3)-begin)/3))) as u32;
                                    let mut val = self.cpu_reader.borrow_mut().read_byte(addr) as i32;

                                    if ch >= 65 && ch <= 70	{// get correct val    
                                        ch -= 7;
                                    }
                                    else if ch >= 97 && ch <= 102 {
                                        ch -= 39;
                                    }
                                    ch -= 48;
                                
                                    if (col % 3) == 1 {		// compute byte val   
                                        val = (ch * 16) + (val % 16);
                                    }
                                    else if (col % 3) == 2 {
                                        val = val - ((val + 16) % 16) + ch;
                                    }
                                    self.cpu_reader.borrow_mut().write_byte(addr, val as u8);
                                }
                                else {
                                    let addr = self.hex_view_offset + ((col-begin) + ((row-1)*((w+1)-begin))) as u32;
                                    self.cpu_reader.borrow_mut().write_byte(addr, ch as u8);
                                }
                                if col < w { col += 1; }
                                else {
                                    if row < h { col = begin; row += 1; }
                                    else { 
                                        col = begin;
                                        let mut r = w-begin;
                                        if hex { r = r/3; }
                                        self.hex_view_offset += (r+1) as u32; 
                                    }
                                }
                                if (col) % 3 == 0 && hex { col += 1; }
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
            }
            0x66 => {
                self.search_label();
            }
            _ => {
                self.refresh_memview();
            }
        }
    }

    ////////////////////////////////////////////
    // static ui create/destroy helper functions
    ////////////////////////////////////////////
    fn create_win(s: &str, h: i32,w: i32,x: i32,y: i32) -> WINDOW {
        let win = newwin(h,w,x,y);
        box_(win,0,0);
        mvwprintw(win,0,1,s);
        wrefresh(win);
        win
    }

    fn destroy_win(win: WINDOW) {
        delwin(win);
    }

    fn create_menu(items : &mut Vec<ITEM>, win : WINDOW, index : u32) -> MENU {
        let mut x = 0;
        let mut y = 0;
        getmaxyx(win,&mut y,&mut x);
        let menu = new_menu(items);
        set_menu_win(menu, win);
        set_menu_sub(menu, derwin(win,y-2,x-2, 1, 1));
        set_menu_format(menu,y-2, 1);

        if index < items.len() as u32 {
            set_current_item(menu, items[index as usize]);
        }
        post_menu(menu);
        wrefresh(win);
        menu
    }

    fn destroy_menu(menu : MENU, items : &mut Vec<ITEM>) {
        unpost_menu(menu);
        for &item in items.iter() {
            free_item(item);
        }
        delwin(menu_sub(menu));
        free_menu(menu);
        drop(menu);
        
        items.clear();//clear/drop of items should be after free of menu, to prevent malloc issues
        drop(items);
    }

    fn string_to_val(val : String) -> u32 {
        if val.len() > 2 && val.as_bytes()[0] == 0x30 && (val.as_bytes()[1] == 0x58 || val.as_bytes()[1] == 0x78) {
            let without_prefix = val.trim_start_matches("0x");
            match u32::from_str_radix(without_prefix, 16) {
                Ok(u) => u,
                Err(_) => 0xDEADBEEF,
            }
        }
        else {
            match u32::from_str_radix(val.as_str(), 10) {
                Ok(u) => u,
                Err(_) => {
                    match u32::from_str_radix(val.as_str(), 16) {
                        Ok(u) => u,
                        Err(_) => 0xDEADDEAD,
                    }
                }
            }
        }
    }
}