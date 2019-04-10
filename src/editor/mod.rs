use ncurses::*;
use crate::virpc::cpu;

static COLOR_BACKGROUND: i16 = 16;
static COLOR_FOREGROUND: i16 = 17;
static COLOR_KEYWORD: i16 = 18;
static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_KEYWORD: i16 = 2;

static WIN1_MAXWIDTH : i32 = 35;
static WIN12_HEIGHT : u32 = 25;
static WIN3_MAXWIDTH : u32 = 30;

pub struct Windows {
    pub items1 : Vec<ITEM>,
    pub items2 : Vec<ITEM>,
    pub items3 : Vec<ITEM>,
    pub menu1 : MENU,
    pub menu2 : MENU,
    pub menu3 : MENU,
    win2_sub : WINDOW,
    win1 : WINDOW,
    win2 : WINDOW,
    win3 : WINDOW,
    win4 : WINDOW,
    screen_height : i32,
    screen_width : i32,
    screen_height_n : i32,
    screen_width_n : i32,
    focus : i32,
    pub edit_pc : u32,
    current_pc : u32,
    pub menu1_choice : u32,
    menu2_choice : u32,
    menu3_choice : u32,
    pub cpu_reader : cpu::CPUShared,
    cur_arg : u32,
}

impl Windows {
    pub fn new(cpu : cpu::CPUShared) -> Windows {

        let mut screen_height = 0;
        let mut screen_width = 0;

        initscr();
        keypad(stdscr(), true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        start_color();
        init_pair(COLOR_PAIR_DEFAULT, COLOR_WHITE, COLOR_BLACK);
        init_pair(COLOR_PAIR_KEYWORD, COLOR_BLACK, COLOR_WHITE);

        refresh();//needed for screen size
        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let s = format!("edit:{:08X},current:{:08X} <F5 run/pause> <F6 reset> <F9 breakpoint> <F10 step>",0,0);
        mvprintw(0,0,s.as_str());
        let lwin1 = Windows::create_win("<commands>",screen_height/2-1, WIN1_MAXWIDTH, 1, 0);
        let lwin2 = Windows::create_win(" code ",screen_height/2-1, screen_width-WIN1_MAXWIDTH, 1, WIN1_MAXWIDTH);
        let lwin3 = Windows::create_win(" variables - arg 0",screen_height/2, screen_width/2, screen_height/2, 0);
        let lwin4 = Windows::create_win(" addressing mode ",screen_height/2, screen_width/2, screen_height/2, screen_width/2);
        let mut litems1 = cpu.borrow_mut().get_commands_list();
        let mut litems2 = cpu.borrow_mut().get_data_list();
        let mut litems3 =  cpu.borrow_mut().get_addressing_mode_list();
        refresh();//needed for win size
        let lmenu1 = Windows::create_menu(&mut litems1,lwin1,0);
        let lmenu2 = Windows::create_menu(&mut litems2,lwin3,0);
        let lmenu3 = Windows::create_menu(&mut litems3,lwin4,0);

        let mut _windows = Windows {
            menu1 : lmenu1,
            menu2 : lmenu2,
            menu3 : lmenu3,
            items1 : litems1,
            items2 : litems2,
            items3 : litems3,
            win2_sub: derwin(lwin2,screen_height/2-3,screen_width-WIN1_MAXWIDTH-2,1,1),
            win1 : lwin1,
            win2 : lwin2,
            win3 : lwin3,
            win4 : lwin4,
            screen_height : screen_height,
            screen_width : screen_width,
            screen_height_n : 0,
            screen_width_n : 0,
            focus : 0,
            edit_pc : 0,
            current_pc : 0,
            menu1_choice : 0,
            menu2_choice : 0,
            menu3_choice : 0,
            cpu_reader :  cpu,
            cur_arg : 0,
        };

        _windows.refresh_pad();
        _windows
    }

    pub fn destroy(&mut self)
    {
        Windows::delete_menu(self.menu1,&mut self.items1);
        Windows::delete_menu(self.menu2,&mut self.items2);
        Windows::delete_menu(self.menu3,&mut self.items3);
        Windows::destroy_win(self.win2_sub);
        Windows::destroy_win(self.win1);
        Windows::destroy_win(self.win2);
        Windows::destroy_win(self.win3);
        Windows::destroy_win(self.win4);
        clear();
        endwin();
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

    fn delete_menu(menu : MENU, items : &mut Vec<ITEM>)
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
            mvwin(self.win1, 1,0);
            wresize(self.win1,self.screen_height/2-1, WIN1_MAXWIDTH);
            box_(self.win1,0,0);
            
            wborder(self.win2, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win2);
            mvwin(self.win2, 1,WIN1_MAXWIDTH);
            wresize(self.win2,self.screen_height/2-1, self.screen_width-WIN1_MAXWIDTH);
            box_(self.win2,0,0);
            
            wborder(self.win3, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win3);
            mvwin(self.win3, self.screen_height/2,0);
            wresize(self.win3,self.screen_height/2, self.screen_width/2);
            box_(self.win3,0,0);
            
            wborder(self.win4, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win4);
            mvwin(self.win4, self.screen_height/2,self.screen_width/2);
            wresize(self.win4,self.screen_height/2, self.screen_width/2);
            box_(self.win4,0,0);

            self.update();
        }
    }

    fn update(&mut self) {
        let s = format!("edit:{:08X},current:{:08X} <F5 run/pause> <F6 reset> <F9 breakpoint> <F10 step>",self.edit_pc,self.current_pc);
        mvprintw(0,0,s.as_str());
        match self.focus {
            0 => {
                mvwprintw(self.win1,0,1,"<commands>");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {}",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
            }
            1 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1,"<code>");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {}",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
            }
            2 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!("<variables> - arg {}",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1," addressing mode ");
            }
            3 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,format!(" variables  - arg {}",self.cur_arg).as_str());
                mvwprintw(self.win4,0,1,"<addressing mode>");
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
    }

    fn refresh_pad(&mut self) {
        let mut lpc = 0;

        let mut end = (self.screen_height/2-3) as u32;
        if (self.edit_pc + (self.screen_height/4-3) as u32) > (self.screen_height/2-3) as u32 {
            end = self.edit_pc + (self.screen_height/4-3) as u32;
        }
        
        wattrset(self.win2_sub, COLOR_PAIR(1));
        wresize(self.win2_sub,self.screen_height/2-3, self.screen_width-WIN1_MAXWIDTH-2);
        wmove(self.win2_sub,0,0);
        for i in 0..end {
            lpc = self.cpu_reader.borrow_mut().disassemble(lpc);
            if i == self.edit_pc {
                wattrset(self.win2_sub, COLOR_PAIR(2));
                
                Windows::delete_menu(self.menu1, &mut self.items1);
                self.items1 = self.cpu_reader.borrow_mut().get_commands_list();
                self.menu1_choice = self.cpu_reader.borrow_mut().get_instruction_index();
                self.menu1 = Windows::create_menu(&mut self.items1, self.win1, self.menu1_choice);

                Windows::delete_menu(self.menu2, &mut self.items2);
                self.items2 = self.cpu_reader.borrow_mut().get_data_list();
                self.menu2_choice = self.cpu_reader.borrow_mut().instruction.arg_index[self.cur_arg as usize];
                self.menu2 = Windows::create_menu(&mut self.items2 ,self.win3, self.menu2_choice);

                Windows::delete_menu(self.menu3, &mut self.items3);
                self.items3 = self.cpu_reader.borrow_mut().get_addressing_mode_list();
                self.menu3_choice = self.cpu_reader.borrow_mut().argument_type(self.cur_arg);
                self.menu3 = Windows::create_menu(&mut self.items3, self.win4, self.menu3_choice);
            }
            if i >= end - ((self.screen_height/2-3) as u32) {
                wprintw(self.win2_sub, self.cpu_reader.borrow_mut().instruction_to_text().as_str());
                wattrset(self.win2_sub, COLOR_PAIR(1));
            }
        }
        wrefresh(self.win2_sub);
    }

    //
    // key event handler
    //
    pub fn handle_keys(&mut self, ch : i32) {
        match ch {
            0x09 => {
                self.focus += 1;
                if self.focus > 3 {
                    self.focus = 0;
                }
                self.update();
            }
            _ => {
                match self.focus {
                    0 => self.handle_keys_win1(ch),
                    1 => self.handle_keys_win2(ch),
                    2 => self.handle_keys_win3(ch),
                    3 => self.handle_keys_win4(ch),
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
            }
            KEY_DOWN => {
                menu_driver(self.menu1, REQ_DOWN_ITEM);
                wrefresh(self.win1);
            }
            _ => {}
        }
    }

    fn handle_keys_win2(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                if self.edit_pc > 0 {
                   self.edit_pc -= 1; 
                }
                self.refresh_pad();
            }
            KEY_DOWN => {
                self.edit_pc += 1;
                self.refresh_pad();
            }
            _ => {}
        }
    }

    fn handle_keys_win3(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                menu_driver(self.menu2, REQ_UP_ITEM);
                wrefresh(self.win3);
            }
            KEY_DOWN => {
                menu_driver(self.menu2, REQ_DOWN_ITEM);
                wrefresh(self.win3);
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
            _ => {}
        }
    }

    fn handle_keys_win4(&mut self, ch : i32) {
        match ch {
            KEY_UP => {
                menu_driver(self.menu3, REQ_UP_ITEM);
                wrefresh(self.win4);
            }
            KEY_DOWN => {
                menu_driver(self.menu3, REQ_DOWN_ITEM);
                wrefresh(self.win4);
            }
            _ => {}
        }
    }
}