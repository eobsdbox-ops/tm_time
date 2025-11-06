/// Working a struct tm in rust without unsafe C for openbsd
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs;

#[derive(Debug,Clone,Copy)]
enum TmZone {
	//Spring = 2nd Sunday in March
	None,Est,Cst,Mst,Pst,Akst,Hst,Sst,
	//Fall = 1st Sunday in November
	Edt,Cdt,Mdt,Pdt,Akdt,Hdt,Sdt,
}

impl TmZone
{
	const PATH_LOCALTIME:&str = "/etc/localtime";
	
	fn get_tzabbr(&self)->&'static str
	{
		
		match self{
			Self::None => "???",
			// Spring = 2nd Sunday in March
			Self::Est  => "EST",  Self::Cst  => "CST",
			Self::Mst  => "MST",  Self::Pst  => "PST",
			Self::Akst => "AKST", Self::Hst  => "HST",
			Self::Sst  => "SST",
			// Fall = 1st Sunday in November
			Self::Edt  => "EDT",  Self::Cdt  => "CDT",
			Self::Mdt  => "MDT",  Self::Pdt  => "PDT",
			Self::Akdt => "AKST", Self::Hdt  => "HDT",
			Self::Sdt  => "SDT",
		}//MATCH
	}//FN
	fn get_utc_offset_sec(&self)->i32
	{
		match self{
			//Spring = 2nd Sunday in March
			Self::None => 0,
			Self::Est  => -18000, // -5
			Self::Cst  => -21600, // -6
			Self::Mst  => -25200, // -7
			Self::Pst  => -28800, // -8
			Self::Akst => -32400, // -9
			Self::Hst  => -36000, // -10
			Self::Sst  => -39600, // -11
			//Fall = 1st Sunday in November
			Self::Edt  => -14400, // -4
			Self::Cdt  => -18000, // -5
			Self::Mdt  => -21600, // -6
			Self::Pdt  => -25200, // -7 
			Self::Akdt => -28800, // -8
			Self::Hdt  => -32400, // -9
			Self::Sdt  => -36000, // -10
			
		}//MATCH
	}//FN
	fn get_utc_offset_hrs(&self)->i8
	{
		(self.get_utc_offset_sec() / 3600) as i8
	}
	fn get_time_zone()->Self
	{
		let timezone:PathBuf = 
			if let Ok(ok) = fs::read_link(&PathBuf::from(&Self::PATH_LOCALTIME)){ ok } else {return TmZone::None};
		let timezone:String = if let Some(s) = timezone.to_str() {String::from(s)}else{return TmZone::None};
		let start:usize = if let Some(s) = timezone.rfind('/'){ s + 1 } else { 0 };
		let tzname = &timezone[start..];
	
		//DST 
		match tzname{
			"Eastern"|"Michigan"|"East-Indiana" => TmZone::Est,
			"Central"|"Indiana-Starke" => TmZone::Cst,
			"Mountain" => TmZone::Mst,
			"Pacific" => TmZone::Pst,
			"Alaska" | "Aleutian" => TmZone::Akst,
			"Hawaii" => TmZone::Hst,
			"Samoa" => TmZone::Sst,
			_ => TmZone::None,
		}
	}
	fn std_to_dst(&mut self)
	{
		match self{
			Self::Est  => *self = Self::Edt,
			Self::Cst  => *self = Self::Cdt,
			Self::Mst  => *self = Self::Mdt,
			Self::Pst  => *self = Self::Pdt,
			Self::Akst => *self = Self::Akdt,
			Self::Hst  => *self = Self::Hdt,
			Self::Sst  => *self = Self::Sdt,
			_ => {}
		}
	}
}//IMPL

#[derive(Debug,Clone,Copy)]
struct TmTime{
	tm_sec:i8,
	tm_min:i8,
	tm_hour:i8,
}

#[derive(Debug,Clone,Copy)]
struct TmDate{
	tm_mon:i8,
	tm_mday:i8,  
	tm_year:u16,
	tm_wday:i8, //Weekday sun - sat
	tm_yday:u16,//Year Day
}

impl TmDate{
	const WEEKDAY_NAME:[&str;7] = [
		"Sunday","Monday",
		"Tuesday","Wednesday",
		"Thursday","Friday",
		"Saturday"
	];
	
	const MONTH_NAME:[&str;12] = [
		"January","Feburay","March",
		"April","May","June","July",
		"August","September","October",
		"November","December"
	];
	
	//Jan 31 Feb 28 Mar 31 Apr 30 May 31 Jun 30 Jul 31 Aug 31 Sep 30 Oct 31 Nov 30 Dec 31
	const MONTHS:[i8;12] = [31,28,31,30,31,30,31,31,30,31,30,31];
	
	fn month_name_abbr(&self)->&str
	{
		let abbr = Self::MONTH_NAME[self.tm_mon as usize];
		&abbr[0..3]
	}
	
	fn week_name_abbr(&self)->&str
	{
		let abbr = Self::WEEKDAY_NAME[self.tm_wday as usize];
		&abbr[0..3]
	}
	
	// fn actual_yr(&self)
	// {
	//	self.tm_year + 1970
	// }
	// Calculates the week number of the year (00-53) with Sunday as the first day of the week.
    // Week 1 begins with the first Sunday in the year. Days before the first Sunday are in week 00.
    fn week_u(&self) -> u16 {
        // tm_yday is 1-based (1 to 366)
        // tm_wday is 0-based (0=Sun to 6=Sat)
        let yday_0based: i32 = self.tm_yday as i32 - 1;
        let wday: i32 = self.tm_wday as i32;
        
        // Standard C library formula for %U: floor((yday_0based + 7 - wday) / 7)
        // This produces the week number 00-53.
        ((yday_0based + 7 - wday) / 7) as u16
    }

    // Calculates the week number of the year (00-53) with Monday as the first day of the week.
    // Week 1 begins with the first Monday in the year. Days before the first Monday are in week 00.
    fn week_w(&self) -> u16 {
        // tm_yday is 1-based (1 to 366)
        // tm_wday is 0-based (0=Sun to 6=Sat)
        
        // Convert the tm_wday (0=Sun, 1=Mon) to a Monday-based index (0=Mon, 1=Tue, ..., 6=Sun)
        let wday_mon0: i32 = (self.tm_wday as i32 + 6) % 7;
        let yday_0based: i32 = self.tm_yday as i32 - 1;

        // Standard C library formula for %W: floor((yday_0based + 7 - wday_mon0) / 7)
        // This produces the week number 00-53.
        ((yday_0based + 7 - wday_mon0) / 7) as u16
    }
	
	fn isoweek_8601(&mut self)->u16
	{
		//Called from weekly
		/* http://www.tondering.dk/claus/cal/node8.html */
		//tm is 0-11 we need 1 - 12
		let tm_mon = self.tm_mon + 1;
		
		let a:i32 = if tm_mon <= 2{ self.tm_year as i32 - 1 }else{self.tm_year as i32};
		let b:i32 = (a/4) - (a/100) + (a/400);
		let c:i32 = ((a-1)/4) - ((a-1)/100) + ((a-1)/400);
		let s:i32 = b - c;
		let e:i32;
		let f:i32;
	
		if tm_mon <= 2{
			e = 0;
			f = self.tm_mday as i32 - 1 + 31 * (tm_mon as i32 -1);
		} else {
			e = s + 1;
			f = self.tm_mday as i32 + ((153 * (tm_mon as i32 -3) + 2) / 5) + 58 + s;
		}
		let g:i32 = (a + b) % 7;
		let d:i32 = (f + g - e) % 7;
		let n:i32 = f + 3 - d;

		if n < 0{
			return (53 - (g - s) / 5) as u16;
		}else if n > 364 + s{
			return 1 as u16;
		}else{ 
			return (n/7 + 1) as u16;
		}
	}
	//year day to month and month day
	fn month_mday(&mut self)
	{
		//Better zero or were going to have problems
		self.tm_mon = 0;
		self.tm_mday = 0;
		
		let mut tm_mday = self.tm_yday;
		for month in TmDate::MONTHS.iter(){
			
			//if month is feburary
			let month:u16 = if self.tm_mon == 1{
					(*month as u8 + leapyrs(self.tm_year as u16)) as u16
				}else{
					*month as u16
			};
			//Have our date and month
			if tm_mday <= month{
				break;
			}
			//
			tm_mday -= month;
			self.tm_mon += 1;
		}
		self.tm_mday = tm_mday as i8;
	}

	//day number of 1 - 365 or 1 - 366
	pub fn day_of_year(&mut self){
	
		// let mut dayofyear:u16 = 0;
	
		if self.tm_mon > 0{
			//Make months 0-11;
			let end:usize = self.tm_mon as usize;
			for days in TmDate::MONTHS[0..end].iter(){
				if *days == 28{
					self.tm_yday += (28 + leapyrs(self.tm_year)) as u16;
				}else{
					self.tm_yday += *days as u16;
				}
			}
		}
		//Add current month
		self.tm_yday += self.tm_mday as u16;
	}

	// 0 - 6 Sun -> Sat
	pub fn weekday(&mut self)
	{
		let mut weekday:u16 = self.year_start_weekday() as u16;
		// weekday += day_of_year(month,day,year);
		if self.tm_yday == 0{
				self.day_of_year();
		}
		weekday += self.tm_yday;
		// weekday += day as u16;
		// Weekdays start at 0 - 6 
		weekday -= 1;
		self.tm_wday = (weekday % 7) as i8;
		
	}

	// Years starting weekday 0 -> 6 Sun -> Sat
	fn year_start_weekday(&self)->u8
	{
		let mut totalday:u32 = 4;
		if self.tm_year > 1970{	
			for yr in 1970..self.tm_year{
				totalday += 365 + (leapyrs(yr) as u32);
			}
		}
		let wday:u32 = totalday % 7;
		wday as u8
	}
	
	// Mon is start 1 - 7
	fn to_monday1_7(&self)->i8
	{
		match self.tm_wday{
			0 => return 7,
			_ => return self.tm_wday,
		}
	}	
}

impl TmTime{
	
	fn to_12hr(&self)->i8
	{
		let mut hour:i8 = self.tm_hour;
		hour %= 12;
		hour += 1;
		hour
	}
	const fn am_pm(&self)->&str
	{
		
		if self.tm_hour < 12{ &"AM" }else{ &"PM" }
	}
}

#[derive(Debug,Clone,Copy)]
struct Tm{
	tm_time:TmTime,
	tm_date:TmDate,
	tm_isdst:bool,
	tm_zone:TmZone,
}

type TimeT = u64;
impl Tm
{
	pub fn gmtime(timecov:TimeT)->Self
	{
		const SEC_IN_MIN:u64 = 60;
		let tm_zone:TmZone = TmZone::None;
		
		//Get Seconds
		let tm_sec:i8 = (timecov % SEC_IN_MIN) as i8;
		
		//Change to mins
		let mut epoch = timecov / 60;
		
		//Get Minutes
		let tm_min:i8 = (epoch % 60) as i8;
		
		//Change to Hrs
		epoch /= 60;
		
		// Get Hour
		let tm_hour:i8 = (epoch % 24) as i8;
		
		let tm_time:TmTime = TmTime{tm_sec,tm_min,tm_hour};
		
		// Change to Days
		epoch /= 24;
		
		// 365 days * 1969 + 0-1969 + 477 leap years
		let epoch_shift:u64 = 719162;
		 
		//Mar+..= Dec = 306
		epoch += epoch_shift + 306;
		
		//  Quarter-days since 0000-02-28 06:00
		let qday:u64 = epoch * 4 + 3;        
		
		// 400 * 365 - 3 = 146097
		//  Century-Februaries elapsed
		let cent:u64 = qday / 146097;        
		
		//  Map to Julian Quarter-Day         
		let qjul:u64 = qday - (cent & !3) + cent * 4;  
		
		//1461 / 4 = 365.25
		//  Year (later incremented if Jan/Feb)
		let mut tm_year:u64 = qjul / 1461;    
		
		//  Day of Year (starting 1 March)
		let mut tm_yday:u64 = (qjul % 1461) / 4;  
		
		//  Neri‑Schneider equivalent of:
		let n:u64 = tm_yday * 2141 + 197913;    
		
		//  m = (yday * 5 + 2) / 153
		// Causes a single-cycle bit operation using 65536
		let m:u64 = n / 65536;                        
		
		//  D = yday - (M * 153 + 2) / 5
		let d:u64 = (n % 65536) / 2141;     
		
		//  Check if Jan or Feb      
		let bump:bool = tm_yday >= 306;               

		let tm_mday:u64 = d + 1;  
		
		//  Compiler optimises to branch-free
		let mut tm_mon:u64 = if bump{ m - 12 } else { m };
		
		// Tm months are 0-11
		tm_mon -= 1;
		
		tm_year += if bump { 1 }else { 0 };
		
		if tm_mon >= 2{
			//add jan
			tm_yday += 31;
			//add feb and leap
			tm_yday += (28 + leapyrs(tm_year as u16)) as u64;
			//add mysterious 1;
			tm_yday += 1;
		}
		
		let tm_mon:i8 = tm_mon as i8;
		let tm_mday:i8 = tm_mday as i8;
		let tm_yday:u16 = tm_yday as u16;
		let tm_year:u16 = tm_year as u16;
		
		let mut tm_date:TmDate = TmDate{tm_mon,tm_mday,tm_year,tm_wday:0,tm_yday};
		
		tm_date.weekday();
		
		Self {
			tm_time,		
			tm_date,
			tm_isdst:false,
			tm_zone,
		}		
	}//FN
	//Call after gmt
	pub fn localetime(&mut self)->&Self
	{	
		self.isdst();
		self		
	}//FN	
	fn mktime(&self)->TimeT{	
		
		let mut tm_yday:i16 = self.tm_date.tm_yday as i16;
		//println!("{tm_yday} = {} as i16",self.tm_yday);
		
		let mut epoch:u64 = self.tm_time.tm_sec as u64; 
		
		//min to hrs
		let secs_in_hrs:u64 = 60 * 60;
		
		//day to secs
		let secs_in_day:u64 = 24 * secs_in_hrs;
		
		//Add minutes
		let mut hold:u64 = self.tm_time.tm_min as u64; 
		epoch += hold * 60;
		//println!("mins_fig:{epoch}");
		
		//Add hours
		hold = self.tm_time.tm_hour as u64;
		epoch += hold * secs_in_hrs; 
		//println!("hours:{tm_hour}\thrs_fig:{epoch}");
		
		// Add days
		// println!("before converting:{tm_yday}");
		
		// Current day is not done
		tm_yday -= 1;
		
		hold = tm_yday as u64;
		// println!("after converting:{hold}");
		epoch += hold * secs_in_day;
		// println!("day_fig:{epoch}");
		
		let end_yr:u16 = self.tm_date.tm_year;
		// println!("end_yr:{end_yr} from {}",self.tm_year);
		
		//Add years
		let mut yrs_total:u64 = 0;
		for yr in 1970..end_yr{
			if leapyrs(yr) == 1{
				//24 hours in day * 60mins * 60 secs
				yrs_total += 366 * secs_in_day;
				continue;
			}
			yrs_total += 365 * secs_in_day;
		}
		//println!("yrs_fig:{yrs_total}");
		epoch += yrs_total;
		epoch
	}
	fn to_gmt(&mut self)->&Self
	{
		let utc:i8 = self.tm_zone.get_utc_offset_hrs() as i8;
		
		//Handle hour underflow
		//Handle hour overflow
		if utc < 0{
			self.utc_hr_add(utc);
		}
		if utc > 0{
			self.utc_hr_sub(utc);
		}
		self
	}
	
	fn utc_hr_sub(&mut self,hrs_dec:i8)
	{
		if hrs_dec < 0{
			self.tm_time.tm_hour += hrs_dec; 
		}else{
			self.tm_time.tm_hour -= hrs_dec; 
		}
		if self.tm_time.tm_hour >= 0 {return} 
		
		self.tm_time.tm_hour += 24;
		self.tm_time.tm_hour %= 24;
		// if self.tm_hour == 0{
		//	return;
		// }
		self.tm_date.tm_mday -= 1;
		if self.tm_date.tm_wday == 0{
			self.tm_date.tm_wday = 7
		}else{
			self.tm_date.tm_wday -= 1;
		}
		//year day - 1 
		if self.tm_date.tm_yday > 0{
			self.tm_date.tm_yday -= 1;
		}else{
			//Jan - 1 = December
			self.tm_date.tm_mon = 12;
			self.tm_date.tm_mday = 31;
			self.tm_date.tm_year -= 1;
			self.tm_date.tm_yday = 365 + leapyrs(self.tm_date.tm_year) as u16;	
			return;
		}
		self.tm_date.month_mday()
	}//FN
	fn utc_hr_add(&mut self,hrs_inc:i8)
	{
		if hrs_inc < 0{
			self.tm_time.tm_hour += (!hrs_inc) + 1;
		}else{
			self.tm_time.tm_hour += hrs_inc;
		}
		if self.tm_time.tm_hour <= 23 {return}
		
		self.tm_time.tm_hour %= 24;
		self.tm_date.tm_mday += 1;
		if self.tm_date.tm_mday > TmDate::MONTHS[self.tm_date.tm_mon as usize] as i8{
			self.tm_date.tm_mon += 1;
			//December + 1 = January 1 year + 1
			if self.tm_date.tm_mon > 11{
				self.tm_date.tm_mon = 0;
				self.tm_date.tm_mday = 1;
				self.tm_date.tm_year += 1;
				self.tm_date.tm_yday = 1;
			}
			else if self.tm_date.tm_mon < 11{
				self.tm_date.tm_mday = 1;
				self.tm_date.tm_yday += 1;
			}
		}else{
			self.tm_date.tm_yday += 1;
		}
	}
	//DST 2nd Sunday in March @ 2am
	fn dst_start(&self)->u8
	{
		//DST
		let yr_start_wday:u8 = self.tm_date.year_start_weekday();
	
		//Add Jan
		let mut dayofyear:u8 = yr_start_wday + 31;
		dayofyear += 28 + leapyrs(self.tm_date.tm_year); 
	
		let mut sunday:u8 = 0;
		loop
		{
			if dayofyear % 7 == 0{
				sunday += 1;
			} 
			if sunday == 2{ 
				break
			}
			dayofyear += 1;
		}
		//Subtract offset of wday
		dayofyear -= yr_start_wday;
		//Month Start at 1
		dayofyear += 1;
		dayofyear as u8
	}
	//DST first Sunday of November @ 2am
	fn dst_end(&self)->u16
	{
		//DST
		let yr_start_wday:u16 = self.tm_date.year_start_weekday() as u16;
		//Jan,Mar,May,Jul,Aug,Oct
		let mut dayofyear:u16 = 31 * 6;  
		//Apr 30 Jun 30 Sep 30
		dayofyear += 30 * 3;
		//Feb
		dayofyear += (28 + leapyrs(self.tm_date.tm_year)) as u16;  
		dayofyear += yr_start_wday;
	
		//Break at 1st Sunday
		loop
		{
			if dayofyear % 7 == 0{
				break;
			} 
			dayofyear += 1;
		}
		dayofyear -= yr_start_wday;
		dayofyear += 1;
		dayofyear
	}
	fn isdst(&mut self) 
	{
		self.tm_zone = TmZone::get_time_zone();	
		let mut offset:i8 = self.tm_zone.get_utc_offset_hrs();
		
		let daynumber:u16 = self.tm_date.tm_yday;
		
		let dst_start:u16 = self.dst_start() as u16;
		let dst_end:u16 = self.dst_end();
		
		//Boolean 
		self.tm_isdst = daynumber >= dst_start && daynumber < dst_end;
		
		if self.tm_isdst {
			
			let dst_day:u16 = self.dst_start() as u16;
			
			if dst_day == self.tm_date.tm_yday{
				
				if self.tm_time.tm_hour >= 2{
					self.tm_zone.std_to_dst();
					offset = self.tm_zone.get_utc_offset_hrs();
				}
			}
			if dst_day < self.tm_date.tm_yday{
				
				self.tm_zone.std_to_dst();
				offset = self.tm_zone.get_utc_offset_hrs();
			}
		}//IF self.tm_isdst
		if offset < 0{
			self.utc_hr_sub(offset);
		}else{
			self.utc_hr_add(offset);
		}
	}
}	

//Formating
impl Tm{
	
	fn strftime(&self,format:&str)->String
	{
		let mut printout:String = String::new();
		let mut set:bool = false;
		
		for ch in format.chars(){
			//Flag
			if ch == '%' && !set{
				set = true;
				continue;
			}
			if !set{
				printout.push(ch);
				continue;
			}
			//AaBbCcDdeFGgHIjklMmnpRrSsTtUuVvWwXxYyZz%+	
			//What is locale? Think I need to read these variables $LC_?
			match ch {
				// Locale full weekday name
				'A' => printout.push_str(&TmDate::WEEKDAY_NAME[self.tm_date.tm_wday as usize]),
				// Locale addreviated weekday name
                'a' => printout.push_str(self.tm_date.week_name_abbr()),
				// locale full month name
                'B' => printout.push_str(&format!("{}",&TmDate::MONTH_NAME[self.tm_date.tm_mon as usize])),
				// locale abbreviated month name                
                'b' | 'h' => printout.push_str(self.tm_date.month_name_abbr()),
				// the century a year divided by 100, 00-99
                'C' => { let year = self.tm_date.tm_year % 100; printout.push_str(&format!("{:02}",year)); }
				// locales appropriate date and time representation
				// Ex:Mon Oct 27 14:33:19 2025
                'c' => printout.push_str(&format!("{} {} {} {}:{}:{} {}", // locale date and time
						self.tm_date.week_name_abbr(),
						self.tm_date.month_name_abbr(),
						self.tm_date.tm_mday,
						self.tm_time.tm_hour,
						self.tm_time.tm_min,
						self.tm_time.tm_sec,
						self.tm_date.tm_year)),
				// date in format "%m/%d/%y"
                'D' => printout.push_str(&format!("{:02}/{:02}/{:02}",
							self.tm_date.tm_mon + 1,
							self.tm_date.tm_mday,
							self.tm_date.tm_year % 100)),
				// day of the month as decimal 01-31
                'd' => printout.push_str(&format!("{:02}",self.tm_date.tm_mday)),
				// day of month as a decimal number 1-31          
                'e' => printout.push_str(&format!("{}",self.tm_date.tm_mday)),
				// date in the format "%Y-%m-%d"       
                'F' => printout.push_str(&format!("{}-{:02}-{:02}",
						self.tm_date.tm_year,
						self.tm_date.tm_mon + 1,
						self.tm_date.tm_mday)),
				// ISO 8601 week-numbering year with centrury as the decimal number
				'G' => printout.push_str(&format!("{:4}",self.tm_date.tm_year)),       
                // ISO 8601 week-numbering year without century as a 00-99
				'g' => printout.push_str(&format!("{:02}",self.tm_date.tm_year % 100)),	
				// 24-hour clock as 00-23
                'H' => printout.push_str(&format!("{:02}",self.tm_time.tm_hour)),
                // 12-hour clock 01-12
                'I' => printout.push_str(&format!("{:02}",self.tm_time.to_12hr())),
                // day of year as decimal 001-366
                'j' => printout.push_str(&format!("{:03}",self.tm_date.tm_yday)),
                // 24-hour clock 0-23
                'k' => printout.push_str(&format!("{}",self.tm_time.tm_hour)),
                // 12-hour clock 1-12
                'l' => printout.push_str(&format!("{}",self.tm_time.to_12hr())),
                // minute as 00-59
                'M' => printout.push_str(&format!("{:02}",self.tm_time.tm_min)),
                // month as 01-12
                'm' => printout.push_str(&format!("{:02}",self.tm_date.tm_mon + 1)), // Month 0-11 to 01-12
                // newline
                'n' => printout.push('\n'),
                // locale equivalent of either "AM" or "PM"
                'p' => printout.push_str(self.tm_time.am_pm()),
                // time in the format "%H:%M"
                'R' => printout.push_str(&format!("{:02}:{:02}",self.tm_time.tm_hour,self.tm_time.tm_min)),
                // 12-hour clock time usine AM/PM notation
                // ex 04:15:19 PM
                'r' => printout.push_str(&format!("{:02}{:02}{:02} {}",
							self.tm_time.to_12hr(),
							self.tm_time.tm_min,
							self.tm_time.tm_hour,
							self.tm_time.am_pm())),
				// seconds as 00-59 	
                'S' => printout.push_str(&format!("{:02}",self.tm_time.tm_sec)),
                // number seconds since the Epoch UTC see mktime(3)
                's' => { 
					let mut tm_local:Tm = self.clone();
					printout.push_str(&format!("{}",tm_local.to_gmt().mktime()));
				}
				// format %H:%M:%S
                'T' => printout.push_str(&format!("{:02}:{:02}:{:02}",
							self.tm_time.tm_hour,
							self.tm_time.tm_min,
							self.tm_time.tm_sec)),
                // tab 
                't' => printout.push('\t'),
                // week number sunday as the 1st day of the week as decimal number 00-53
				// 1st Sunday Week numbering
				'U' => printout.push_str(&format!("{:02}",self.tm_date.week_u())),
				// weekday Monday as the 1st day of week as 1-7
				// monday = 1;
				'u' => printout.push_str(&format!("{}",self.tm_date.to_monday1_7())),
				// week number of year Monday as the first day of week as 01-53. 
				// if week containing Jan 1st has four or more days in the new year this it 
				// is week 1 otherwise it is week 53 of previous year and the next week is
				// week 1 The year is given by the %G conversion specification.
				'V' => { let mut tm_local:TmDate = self.tm_date;
						 printout.push_str(&format!("{}",tm_local.isoweek_8601()));
				}
				// date format "%e-%b-%Y"
                'v' => printout.push_str(&format!("{}-{}-{}",
							self.tm_date.tm_mday,
							self.tm_date.month_name_abbr(),
							self.tm_date.tm_year,
                    )),
                
                // week number of year Monday 1st day of week. 00-53
				// Not Standard
				'W' => printout.push_str(&format!("{:02}",self.tm_date.week_w())),
				// weekday sunday 1st day of week as 0-6
                'w' => printout.push_str(&format!("{}",self.tm_date.tm_wday)),
                // locale's appropriate time representation
				// ex 16:22:50
				'X' => printout.push_str(&format!("{}:{}:{}",
						self.tm_time.tm_hour,
						self.tm_time.tm_min,
						self.tm_time.tm_sec)),
				// locale's appropriate date representation  
				// ex 10/27/25
				'x' => printout.push_str(&format!("{}/{}/{:02}",
							self.tm_date.tm_mon,
							self.tm_date.tm_mday,
							self.tm_date.tm_year % 100)),
				// year with century as number
                'Y' => printout.push_str(&format!("{}",self.tm_date.tm_year)),
                // year without century as a decimal number 00--99
                'y' => printout.push_str(&format!("{:02}",self.tm_date.tm_year % 100)),
                // time zone name or by the empty string 
                'Z' => printout.push_str(self.tm_zone.get_tzabbr()), // Used TmZone abbreviation
                // offset from UTC in the format "+HHMM" or "-HHMM" or empty string
                'z' => {
                    let offset_hrs = self.tm_zone.get_utc_offset_hrs();
                    let sign = if offset_hrs <= 0 { '-' } else { '+' };
                    let abs_offset = offset_hrs.abs();
                    printout.push_str(&format!("{}{:02}00", sign, abs_offset));
                }
                // % character
                '%' => printout.push('%'),
                // FIX: Extract slices to local variables to satisfy the `Sized` bound required by format!
                // Ex: Wed Oct 15 07:43:29 EDT 2025
                '+' => {
                    let wday_abbr = self.tm_date.week_name_abbr();
                    let mon_abbr = self.tm_date.month_name_abbr();
                    printout.push_str(&format!("{} {} {} {:02}:{:02}:{:02} {} {}",
                        wday_abbr,
                        mon_abbr,
                        self.tm_date.tm_mday,
                        self.tm_time.tm_hour, self.tm_time.tm_min, self.tm_time.tm_sec,
                        self.tm_zone.get_tzabbr(),
                        self.tm_date.tm_year
                    ));
                }
				_ => printout.push(ch),
			}//Match
			set = false;
		}//For
		print!("{}",printout);
		printout
	}
}

pub fn now()->TimeT
{
	match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(n) => return n.as_secs(),
		Err(_) => return 0,
	}
}
fn leapyrs(year:u16)->u8{
	if(year % 4 == 0 && year % 100 != 0) || (year % 400 == 0){
		return 1;
	}else{
		return 0;
	};
}


fn main()
{
	
	let clock1:TimeT = now();
	//let clock1:TimeT = !0;
	let mut tm_local:Tm = Tm::gmtime(clock1);
	dbg!(&tm_local);
	tm_local.localetime();
	dbg!(&tm_local);
	let clock2:TimeT = tm_local.to_gmt().mktime();
	println!("{}",clock1 - clock2);
}
