  
#![allow(dead_code)]
#[allow(unused_imports)]
#[allow(snake_case)]
extern crate minifb;
extern crate byteorder;
extern crate num;
extern crate ncurses;

#[macro_use]
extern crate enum_primitive;

#[macro_use]
mod utils;
mod virpc;
mod debugger;

use minifb::*;
use std::env;
use ncurses::*;

static COLOR_BACKGROUND: i16 = 16;
static COLOR_FOREGROUND: i16 = 17;
static COLOR_KEYWORD: i16 = 18;
static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_KEYWORD: i16 = 2;

pub struct windows {
    items1 : Vec<ITEM>,
    items2 : Vec<ITEM>,
    items3 : Vec<ITEM>,
    menu1 : MENU,
    menu2 : MENU,
    menu3 : MENU,
    pad : WINDOW,
    win1 : WINDOW,
    win2 : WINDOW,
    win3 : WINDOW,
    win4 : WINDOW,
    screen_height : i32,
    screen_width : i32,
    screen_height_n : i32,
    screen_width_n : i32,
    focus : i32,
    px : i32,
    py : i32,
}

impl windows {
    pub fn new() -> windows {
        let mut screen_height = 0;
        let mut screen_width = 0;

        let mut litems1: Vec<ITEM> = Vec::new();
        litems1.push(new_item("Load", ""));
        litems1.push(new_item("Store", ""));
        litems1.push(new_item("Add", ""));
        litems1.push(new_item("Subtract", ""));
        litems1.push(new_item("Jump", ""));

        let mut litems2: Vec<ITEM> = Vec::new();
        litems2.push(new_item("new local  (stack)", "1"));
        litems2.push(new_item("new global (heap)", "2"));
        litems2.push(new_item("-existing-", "3"));
        litems2.push(new_item(" l_var1", "4"));
        litems2.push(new_item(" g_var2", "5"));

        let mut litems3: Vec<ITEM> = Vec::new();
        litems3.push(new_item("new label", ""));
        litems3.push(new_item("-existing-", ""));
        litems3.push(new_item(" label_0x0001", ""));
        litems3.push(new_item(" label_0xffff", ""));
        litems3.push(new_item(" lib_printf(a)", ""));

        getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
        let mut lwin1 = windows::create_win("<commands>",screen_height/2, screen_width/2, 0, 0);
        let mut lwin2 = windows::create_win(" code ",screen_height/2, screen_width/2, 0, screen_width/2);
        let mut lwin3 = windows::create_win(" variables ",screen_height/2, screen_width/2, screen_height/2, 0);
        let mut lwin4 = windows::create_win(" labels ",screen_height/2, screen_width/2, screen_height/2, screen_width/2);

        let mut lmenu1 = windows::create_menu(&mut litems1,lwin1);
        let mut lmenu2 = windows::create_menu(&mut litems2,lwin3);
        let mut lmenu3 = windows::create_menu(&mut litems3,lwin4);
        //unpost_menu(lmenu1); TODO: check if this is leaky
        set_menu_mark(lmenu1, ">");
        post_menu(lmenu1);
        wrefresh(lwin1);
        let mut _windows = windows {
            menu1 : lmenu1,
            menu2 : lmenu2,
            menu3 : lmenu3,
            items1 : litems1,
            items2 : litems2,
            items3 : litems3,
            pad : newpad(100,100),
            win1 : lwin1,
            win2 : lwin2,
            win3 : lwin3,
            win4 : lwin4,
            screen_height : screen_height,
            screen_width : screen_width,
            screen_height_n : 0,
            screen_width_n : 0,
            focus : 0,
            px : 0,
            py : 0,
        };
        //_windows.win2 = subpad(_windows.pad,screen_height/2, screen_width/2, 0, 0);
        for i in 1..10000 {
            pechochar(_windows.pad, (0x20 + i) as u64);
        }
        prefresh(_windows.pad,0,0,1,screen_width/2+1,screen_height/2-2,screen_width-3);
        _windows
    }
    pub fn resize_check(&mut self,x:i32,y:i32) {
        getmaxyx(stdscr(), &mut self.screen_height_n, &mut self.screen_width_n);
        if self.screen_height != self.screen_height_n || self.screen_width != self.screen_width_n {
            self.screen_height = self.screen_height_n;
            self.screen_width = self.screen_width_n;
            let ch = ' ' as chtype;
            wborder(self.win1, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win1);
            mvwin(self.win1, 0,0);
            wresize(self.win1,self.screen_height/2, self.screen_width/2);
            box_(self.win1,0,0);
            
            wborder(self.win2, ch, ch, ch, ch, ch, ch, ch, ch);
            wrefresh(self.win2);
            mvwin(self.win2, 0,self.screen_width/2);
            wresize(self.win2,self.screen_height/2, self.screen_width/2);
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
            
            self.px = x;
            self.py = y;

            self.update();
        }
    }
    
    pub fn focus(&mut self, focus : i32) {
        self.focus = focus;
        self.update();
    }

    pub fn scrollpad(&mut self, x : i32, y : i32) {
        self.px = x;
        self.py = y;
        self.update();
    }

    pub fn destroy(&mut self)
    {
        windows::delete_menu(self.menu1,&mut self.items1);
        windows::delete_menu(self.menu2,&mut self.items2);
        windows::delete_menu(self.menu3,&mut self.items3);
        clear();
        endwin();
    }

    fn destroy_win(win: WINDOW) {
        delwin(win);
    }

    fn create_win(s: &str, h: i32,w: i32,x: i32,y: i32) -> WINDOW {
        let win = newwin(h,w,x,y);
        box_(win,0,0);
        mvwprintw(win,0,1,s);
        wrefresh(win);
        win
    }

    fn update(&mut self) {
        match self.focus {
            0 => {
                mvwprintw(self.win1,0,1,"<commands>");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1," variables ");
                mvwprintw(self.win4,0,1," labels ");
            }
            1 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1,"<code>");
                mvwprintw(self.win3,0,1," variables ");
                mvwprintw(self.win4,0,1," labels ");
            }
            2 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1,"<variables>");
                mvwprintw(self.win4,0,1," labels ");
            }
            3 => {
                mvwprintw(self.win1,0,1," commands ");
                mvwprintw(self.win2,0,1," code ");
                mvwprintw(self.win3,0,1," variables ");
                mvwprintw(self.win4,0,1,"<labels>");
            }
            _ => {
                
            }
        }

        refresh();            
        wrefresh(self.win1);
        wrefresh(self.win2);
        wrefresh(self.win3);
        wrefresh(self.win4);
        prefresh(self.pad,self.py,self.px,1,self.screen_width/2+1,self.screen_height/2-2,self.screen_width-3);
    }

    fn create_menu(items : &mut Vec<ITEM>, win : WINDOW) -> MENU {
                /* Crate menu */
        let my_menu = new_menu(items);
        menu_opts_off(my_menu, O_SHOWDESC);
        //menu_opts_off(my_menu, O_ONEVALUE);
        /* Set main window and sub window */
        set_menu_win(my_menu, win);
        set_menu_sub(my_menu, derwin(win, 7, 0, 1, 1));
        set_menu_format(my_menu, 7, 1);

        /* Set menu mark to the string " * " */
        set_menu_mark(my_menu, " ");

        /* Post the menu */
        post_menu(my_menu);
        wrefresh(win);
        my_menu
    }

    fn delete_menu(my_menu : MENU, items : &mut Vec<ITEM>)
    {
        unpost_menu(my_menu);

        /* free items */
        for &item in items.iter() {
        free_item(item);
        }

        free_menu(my_menu);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut prg_to_load  = String::new();
    let mut debugger_on  = true;//false;
    let mut window_scale = Scale::X2;


    // process cmd line params
    for i in 1..args.len() {
        if args[i] == "debugger" {
            debugger_on = true;
        }
        else if args[i] == "x2" {
            window_scale = Scale::X2;
        }
        else if args[i].ends_with(".prg") {
            prg_to_load = args[i].clone();
        }
    }

    if prg_to_load == "" {
        prg_to_load = "test.prg".to_string();
    }

    //let mut virpc = virpc::Virpc::new(window_scale, debugger_on, &prg_to_load);
    //virpc.reset();

    // main update loop
    //while virpc.main_window.is_open() {
    //    virpc.run();
    //}

    initscr();
    keypad(stdscr(), true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_color(COLOR_BACKGROUND, 0, 43 * 4, 54 * 4);
    init_color(COLOR_FOREGROUND, 142 * 4, 161 * 4, 161 * 4);
    init_pair(COLOR_PAIR_DEFAULT, COLOR_FOREGROUND, COLOR_BACKGROUND);
    init_pair(COLOR_PAIR_KEYWORD, COLOR_KEYWORD, COLOR_BACKGROUND);
    /* Set the window's background color. */
    //bkgd(' ' as chtype | COLOR_PAIR(COLOR_PAIR_DEFAULT) as chtype);
    refresh();
    let mut _windows : windows = windows::new();
    refresh();

    let mut x = 0;
    let mut y = 0;
    let mut ch = getch();
    let mut focus = 0;
    while ch != 27 as i32 { // ESC pressed

        match ch {
            KEY_UP => {
                //mvwprintw(_windows.win1, 1,1,"test");
                menu_driver(_windows.menu1, REQ_UP_ITEM);
                wrefresh(_windows.win1);
            }
            KEY_DOWN => {
                //mvwprintw(_windows.win1, 1,1,"test");
                menu_driver(_windows.menu1, REQ_DOWN_ITEM);
                wrefresh(_windows.win1);
            }
            KEY_LEFT => {
                x -= 1;
                _windows.scrollpad(x,y);
            }
            KEY_RIGHT => {
                x += 1;
                _windows.scrollpad(x,y);
            }
            KEY_TAB => {
                focus += 1;
                if focus > 3 {
                    focus = 0;
                }
                _windows.focus(focus);
            }
            _ => {

            }
        }
        _windows.resize_check(0,0);
        ch = getch();
    }
    _windows.destroy();
}
