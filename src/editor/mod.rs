use ncurses::*;
use crate::virpc::cpu;

static COLOR_BACKGROUND: i16 = 16;
static COLOR_FOREGROUND: i16 = 17;
static COLOR_KEYWORD: i16 = 18;
static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_KEYWORD: i16 = 2;
static MEMORY_SIZE: u32 = 0x080000;

//TODO define labels: has name, location, size, and highlight current in hexview
//TODO add search for label in code
//TODO add new label in code
//TODO add search for label in mem/bss (and scroll to current label)
//TODO add new label in mem/bss
//TODO add whole memview(separate window)
//TODO add colorised modified values in hexview
//TODO map PC to mem-location
//TODO add custom handling of ldr/str arguments, and argument printing

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
    mem_address : u32,
}

impl Windows {
    pub fn new(cpu : cpu::CPUShared) -> Windows {

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
            mem_address : 0,
        };

        initscr();
        keypad(stdscr(), true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        start_color();
        init_pair(COLOR_PAIR_DEFAULT, COLOR_WHITE, COLOR_BLACK);
        init_pair(COLOR_PAIR_KEYWORD, COLOR_BLACK, COLOR_WHITE);

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
        refresh();//needed for win size
        win.menu1 = Windows::create_menu(&mut win.items1,win.win1,0);
        win.menu2 = Windows::create_menu(&mut win.items2,win.win3,0);
        win.menu3 = Windows::create_menu(&mut win.items3,win.win4,0);
        
        win.screen_height = 0;
        win.screen_width = 0;
        win.resize_check();
        win
    }

    pub fn destroy(&mut self)
    {
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
                    _ => { panic!("dimension index not defined") },
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
                    _ => { panic!("dimension index not defined") },
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
                    _ => { panic!("dimension index not defined") },
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
                    _ => { panic!("dimension index not defined") },
                }
            }
            5 => {
                match dimension {
                    'y' => { sh/2 }, 
                    'x' => { WIN1_MAXWIDTH },
                    'h' => { (sh/2) },
                    'w' => { sw-WIN1_MAXWIDTH },
                    'u' => { sh },
                    'v' => { sw },
                    _ => { panic!("dimension index not defined") },
                }
            }
            _ => { panic!("window index not defined") },
        }

    }

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

    fn destroy_menu(menu : MENU, items : &mut Vec<ITEM>)
    {
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

            self.update();
        }
    }

    fn update(&mut self) {
        let s = format!("edit:{:08X},current:{:08X} <F5 run/pause> <F6 reset> <F9 breakpoint> <F10 step>",self.edit_line,self.current_pc);
        mvprintw(0,0,s.as_str());
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

        self.refresh_pad();

        Windows::destroy_menu(self.menu1, &mut self.items1);
        self.items1 = self.cpu_reader.borrow_mut().get_commands_list();
        self.menu1 = Windows::create_menu(&mut self.items1, self.win1, self.menu1_choice);

        Windows::destroy_menu(self.menu2, &mut self.items2);
        self.items2 = self.cpu_reader.borrow_mut().get_data_list();
        self.menu2 = Windows::create_menu(&mut self.items2 ,self.win3, self.menu2_choice);

        Windows::destroy_menu(self.menu3, &mut self.items3);
        self.items3 = self.cpu_reader.borrow_mut().get_addressing_mode_list();
        self.menu3 = Windows::create_menu(&mut self.items3, self.win4, self.menu3_choice);

        self.mem_address = self.cpu_reader.borrow_mut().get_data_value(self.menu2_choice);
        self.refresh_memview();
    }

    fn refresh_pad(&mut self) {
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
            if i >= end - ((self.screen_height/2-3) as u32) {
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
        for i in 0..(self.wd(5,'h')-2) {
            let mut hex = "".to_string();
            let mut asci = "".to_string();
            let w = (self.wd(5,'w')/4)-4;
            if ( self.mem_address + (i*w) as u32 )  >= MEMORY_SIZE { break; }
            for j in 0..w {
                let adr = self.mem_address + ( (i*w) + j ) as u32;
                let val : u8 = self.cpu_reader.borrow_mut().read_byte(adr);
                if adr < MEMORY_SIZE {
                    hex = format!("{} {:02X}",hex,val);
                    asci = format!("{}{}",asci,EBCDIC[val as usize]);
                }
                else {
                    hex = format!("{}   ",hex);
                    asci = format!("{} ",asci);
                }
            }
            let s = format!("${:08X} |{} | {}",self.mem_address + (i*w) as u32,hex,asci);
            mvwprintw(self.win5,i+1,1,s.as_str());
        }
        wrefresh(self.win5);
    }

    fn modify(&mut self) {
        //take all current settings from menu 1, 2 and 3
        self.cpu_reader.borrow_mut().set_opcode(self.edit_cmd, self.edit_mode[0], self.edit_mode[1], self.edit_mode[2]);
        //from arg, take 1, 2 and 3 based on size
        self.cpu_reader.borrow_mut().parse_args(self.edit_item[0],self.edit_item[1],self.edit_item[2]);
        //write bytecode
        self.cpu_reader.borrow_mut().assemble();
    }

    fn new_val(&mut self) {
        //menu for a new value based on argument-index (self.cur_arg) 
        //new window, asking to input a value
        let mut screen_height = 0;
        let mut screen_width = 0;
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let lwin_menu = Windows::create_win(" ",screen_height/2, screen_width/2, screen_height/4, screen_width/4);

        let s;
        let v;
        let d = " ".to_string();

        //and opcode: edit_cmd or (self.cpu_reader.borrow_mut().instruction_u8)
        let mut code : u8;
        if self.edit_cmd != -1 { code = self.edit_cmd as u8; }
        else { code = self.cpu_reader.borrow_mut().instruction_u8 >> 4; }
        code &= 0x0f;

        match self.edit_item[self.cur_arg as usize] {
            0 => {
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

                            self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_Item(s, d, v) ) as i32;
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
            }
            1 if self.cur_arg == 1 && code == 0x00 => {//jmp
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
                            s = format!("CONST_{}",select);
                            v = select as u32;
                            self.edit_mode[1] = 0;
                            self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_Item(s, d, v) ) as i32;
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
            }/*
            1 if self.cur_arg == 1 && code == 0x01 => {//call or jmp
                //dialog for call: select an address (provide suggestions, based on labels)
                call cpu for list of labels in range code
                OR input value (and add new label)
                BOTH update code for help
            }
            2 => {
                //dialog for new heap: select an address (provide suggestions, based on labels)
                call cpu for list of labels in range mem
                OR input value (and add new label)
                BOTH update memview for help
            }
            3 => {
                //dialog for BSS: select an address (provide suggestions based on labels)
                call cpu for list of labels in range bss
                OR input value (and add new label)
                BOTH update memview for help
            }


            2 if self.cur_arg == 1 && code == 0x0a  => {//ldr
                //edit cpu-arg and opcode if necesary
                //self.edit_cmd, self.edit_mode[0], self.edit_mode[1], self.edit_mode[2]
            }
            2 if self.cur_arg == 2 && code == 0x0a=> {//ldr
            }
            2 if self.cur_arg == 1 && code == 0x0b=> {//str
            }
            2 if self.cur_arg == 2 && code == 0x0b=> {//str
            }*/
            _ => {
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
                            if val.len() > 2 && val.as_bytes()[0] == 0x30 && (val.as_bytes()[1] == 0x58 || val.as_bytes()[1] == 0x78) {
                                let without_prefix = val.trim_left_matches("0x");
                                v = u32::from_str_radix(without_prefix, 16).unwrap();
                            }
                            else {
                                v = u32::from_str_radix(val.as_str(), 10).unwrap();
                            }
                            s = format!("CONST_{}",v);
                            self.edit_item[self.cur_arg as usize] = cpu::CPU::add_new_item(&mut self.cpu_reader.borrow_mut().data, cpu::CPU::new_Item(s, d, v) ) as i32;
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
            },
        }

        Windows::destroy_win(lwin_menu);
        self.edit_item[self.cur_arg as usize] = -1;
    }

    fn reset_edit(&mut self) {
        self.edit_cmd = -1;
        self.edit_item[0] = -1;
        self.edit_item[1] = -1;
        self.edit_item[2] = -1;
        self.edit_mode[0] = -1;
        self.edit_mode[1] = -1;
        self.edit_mode[2] = -1;
    }
    //
    // key event handler
    //
    pub fn handle_keys(&mut self, ch : i32) {
        match ch {
            0x09 => {
                self.focus += 1;
                if self.focus > 4 {
                    self.focus = 0;
                }
                self.update();
            }
            0xa => {
                if self.edit_item[self.cur_arg as usize] > -1 && self.edit_item[self.cur_arg as usize] < 4 {
                    self.new_val();
                }
                else {
                    self.modify();//edit the current value
                }
                self.screen_height = 0;//trigger an update
                self.resize_check();//show the edited value
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
                self.update();
            }
            KEY_DOWN => {
                self.edit_line += 1;
                self.reset_edit();
                self.update();
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
                    self.update();
                }
            }
            KEY_RIGHT => {
                if self.cur_arg < 2 {
                    self.cur_arg += 1;
                    self.update();                    
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
                if self.mem_address >= w {
                    self.mem_address -= w;
                }
                else {
                    self.mem_address = 0;
                }
                self.refresh_memview();
            }
            KEY_DOWN => {
                if self.mem_address < MEMORY_SIZE-1 {
                    self.mem_address += w;
                }
                self.refresh_memview();
            }            
            0x65 => {
                self.refresh_memview();
                curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
                
                let mut hex = true;
                let mut begin : i32 = 13;
                let mut col = begin;
                let mut row = 1;
                let mut w = ((self.wd(5,'w')/4)*3)-1;
                let h = self.wd(5,'h') - 2;

                let mut ch = 0;
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
                                    col = w;
                                    let mut r = w-begin;
                                    if hex { r /= 3; }
                                    if self.mem_address as i32 - r >= 0 { self.mem_address -= (r+1) as u32; }
                                    else { self.mem_address = 0; }
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
                                    self.mem_address += (r+1) as u32; 
                                }
                            }

                            if (col) % 3 == 0 && hex { col += 1; }
                        }
                        KEY_UP => {
                            let mut r = w-begin;
                            if hex { r /= 3; }
                            if row > 1 { row -= 1; }  
                            else {
                                if self.mem_address as i32 - (r+1) >= 0 { 
                                    self.mem_address -= (r+1) as u32; 
                                }
                                else { self.mem_address = 0; }
                            }
                        }
                        KEY_DOWN => {
                            if row < h { row += 1; }
                            else { 
                                let mut r = w-begin;
                                if hex { r /= 3; }
                                self.mem_address += (r+1) as u32; 
                            }
                        }
                        0x9 => {
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
                                    let addr = self.mem_address + (((col-begin)/3) + ((row-1)*(((w+3)-begin)/3))) as u32;
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
                                    let addr = self.mem_address + ((col-begin) + ((row-1)*((w+1)-begin))) as u32;
                                    self.cpu_reader.borrow_mut().write_byte(addr, ch as u8);
                                }
                                if col < w { col += 1; }
                                else {
                                    if row < h { col = begin; row += 1; }
                                    else { 
                                        col = begin;
                                        let mut r = w-begin;
                                        if hex { r = r/3; }
                                        self.mem_address += (r+1) as u32; 
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
            _ => {
                self.refresh_memview();
            }
        }
    }
}