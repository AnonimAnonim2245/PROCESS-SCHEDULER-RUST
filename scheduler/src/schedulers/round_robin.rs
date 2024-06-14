use std::collections::VecDeque;
use std::num::NonZeroUsize;
use crate::scheduler::Process;
use crate::scheduler::Syscall;
use crate::scheduler::MyProcess;
use crate::{Scheduler, SchedulingDecision, Pid, SyscallResult, StopReason};
use crate::ProcessState as OtherProcessState;
/// The state of a process.

pub struct RoundRobin{
    processes: VecDeque<MyProcess>,
    running_process: MyProcess,
    timeslice: NonZeroUsize,
    initial_timeslice: usize,
    next_pid: usize,
    minimum_remaining_timeslice: usize,
    remaining_time : usize,

    
}
/// indexul va fi mereu decat lungimea lui MyProcess ca este updatat la fiecare pas
/// deci nu are cum sa dea panic la un wrap
impl RoundRobin{
    fn syncronize_timings(&mut self,timeslice : usize){
        let mut index = 0;
        
        let mut processes = self.processes.clone();
        for process2 in &mut processes {
            if process2.state == OtherProcessState::Running{
                if index<self.processes.len(){
                   
                    self.processes.get_mut(index).unwrap().timings.0 += timeslice;
                    self.processes.get_mut(index).unwrap().timings.2+= timeslice;
                }
            }
            else{
                self.processes.get_mut(index).unwrap().timings.0 += timeslice;
                
                if(process2.state == OtherProcessState::Waiting{event: None}){
                    if timeslice >= process2.sleep_time {

                        if index+1<self.processes.len() && index<self.processes.len(){
                        self.processes.get_mut(index).unwrap().timings.0 -= timeslice;
                        self.processes.get_mut(index+1).unwrap().timings.0 += timeslice;
                                self.processes.get_mut(index).unwrap().sleep_time = 0;
                                self.processes.get_mut(index).unwrap().state = OtherProcessState::Ready;
                                self.processes.swap(index,index+1);
                        }
                        else{
                            if index< self.processes.len(){
                           
                            self.processes.get_mut(index).unwrap().sleep_time = 0;
                            self.processes.get_mut(index).unwrap().state = OtherProcessState::Ready;
                            }
                        }
                        
                    } else {
                        if index<self.processes.len(){
                        self.processes.get_mut(index).unwrap().sleep_time -= timeslice;
                        }
                    }
                }

                

            }
            index+=1;
           
        }
        
    }
    fn increment_timings(&mut self){
        for process in &mut self.processes {
            
            process.timings.0 += 1;
           
        }
    }
    fn check_number_1_pid(&mut self) -> bool{
        for process in &mut self.processes {
            if process.pid()==1{
                return true;
            }
        }
       false
    }
    fn find_ready_process(&mut self) -> Option<MyProcess>{
        for process in &mut self.processes {
            if process.state == OtherProcessState::Ready{
                return Some(process.clone());
            }
        }
         None
    }
    fn find_running_process(&mut self) -> Option<MyProcess>{
        for process in &mut self.processes {
            if process.state == OtherProcessState::Running{
                return Some(process.clone());
            }
        }
         None
    }
    fn find_sleep_process(&mut self) -> Option<MyProcess>{
        for process in &mut self.processes {
            if(process.state == OtherProcessState::Waiting{event: None}){
                return Some(process.clone());
            }
        }
         None
    }
    pub fn new(timeslice: NonZeroUsize, minimum_remaining_timeslice: usize) -> RoundRobin {
        print!("???");
        RoundRobin {
            processes: VecDeque::new(),
            timeslice,
            running_process: MyProcess::new(1),
            next_pid: 1,
            minimum_remaining_timeslice,
            remaining_time : timeslice.into(),
            initial_timeslice: timeslice.into(),
        }

    }

}

impl Scheduler for RoundRobin {

    
    fn next(&mut self) -> SchedulingDecision {
    
        let size = self.processes.len();
            let is_pid_1_valid = {
            self.check_number_1_pid()
        };
        
        
        if size==0{
           return SchedulingDecision::Done;
        }
         
        if !is_pid_1_valid{
            return SchedulingDecision::Panic;
        }
       
        if self.find_ready_process().is_none() && self.find_running_process().is_none()
        && self.find_sleep_process().is_none() && !self.processes.is_empty() {
                        
                return SchedulingDecision::Deadlock;
        }
        if let Some(mut process) = self.processes.pop_front() {
                
                let pide = process.pid();
             
                if !process.is_sleeping && process.sleep_time!=0{
                    process.timings.0+=process.sleep_time;
                    self.syncronize_timings(process.sleep_time);
                    process.sleep_time=0;
                    if self.initial_timeslice>0 {
                    self.timeslice = std::num::NonZeroUsize::new(self.initial_timeslice).unwrap();
                    }
                    process.state = OtherProcessState::Running;
                    self.processes.push_front(process.clone());
                   
                    return SchedulingDecision::Run{pid: pide,timeslice:self.timeslice};


                    
                }
                
                if !is_pid_1_valid{
                    self.processes.push_front(process.clone());

                    return SchedulingDecision::Panic;
                }
                if self.timeslice.get() >= self.minimum_remaining_timeslice && process.state == OtherProcessState::Running {
                    self.processes.push_front(process.clone());
                    
                   SchedulingDecision::Run{pid: pide,timeslice:self.timeslice}
                }
                else{
                    let process_pid = process.pid();
                    

                    if process.state == OtherProcessState::Ready{
                        process.state = OtherProcessState::Running;
                        self.processes.push_back(process.clone());
                        
                    }
                    else if self.find_ready_process().is_none(){
                        self.processes.push_front(process.clone());
                        
                        if let Some(mut process) = self.find_running_process(){
                            process.state = OtherProcessState::Running;
                            if self.initial_timeslice>0 {
                              self.timeslice = std::num::NonZeroUsize::new(self.initial_timeslice).unwrap();
                            }
                            let mut element: VecDeque<MyProcess> = VecDeque::new();
                            for p in &self.processes {
                                if p.pid()!=process.pid(){
                                    element.push_back(p.clone());
                                }
                                                                
                            }
                            element.push_front(process.clone());
                            self.processes = element;
                           
                            return SchedulingDecision::Run{pid: process.pid(),timeslice:self.timeslice};

                        }

                        if let Some(mut process)=self.find_sleep_process(){
                            let mut value:VecDeque<MyProcess> = VecDeque::new();

                            for p in &mut self.processes {
                               
                                if p.pid()!=process.pid(){
                                    value.push_back(p.clone());
                                }
                            }
                            self.processes = value;
                            if(process.state==OtherProcessState::Waiting{event: None}){
                                if process.sleep_time>0 {
                                self.timeslice = std::num::NonZeroUsize::new(process.sleep_time).unwrap();
                                }
                                process.is_sleeping=false;
                                self.processes.push_front(process.clone());
                              
                                                    
                               
                                        
                                if process.sleep_time>0{
                                return SchedulingDecision::Sleep( std::num::NonZeroUsize::new(process.sleep_time()).unwrap());
                                }
                            }
                            process.state = OtherProcessState::Running;
                            self.processes.push_back(process.clone());
                        }
                        else{
                            
                            return SchedulingDecision::Deadlock;

                        }
                          
                    }
                    else{
                        if(process.state != OtherProcessState::Waiting { event: (None) } && process.state != OtherProcessState::Waiting{event:Some(process.event_number)}){
                        process.state= OtherProcessState::Ready;}
                        
                        if self.find_running_process().is_none(){
                            if let Some(mut process2) = self.processes.pop_front(){
                                self.processes.push_back(process.clone());
                                while process2.state != OtherProcessState::Ready{
                                    self.processes.push_back(process2.clone());
                                    if !self.processes.is_empty(){
                                    process2 = self.processes.pop_front().unwrap();
                                    }
                                }

                                if self.initial_timeslice>0 {
                                    self.timeslice = std::num::NonZeroUsize::new(self.initial_timeslice).unwrap();
                                }
                                process2.state = OtherProcessState::Running;
                                self.processes.push_front(process2.clone());
                                
                                return SchedulingDecision::Run{pid: process2.pid(),timeslice:self.timeslice};
                                
                            }
                         }
                         else{
                            
                            self.processes.push_back(process.clone());
                          
                            // aici verifica daca exista sau nu un process running, deci nu da panic
                            process = self.find_running_process().unwrap();

                            if self.initial_timeslice>0{
                            self.timeslice = std::num::NonZeroUsize::new(self.initial_timeslice).unwrap();
                            }

                            
                            return SchedulingDecision::Run{pid: process.pid(),timeslice:self.timeslice};

                            

                         }
                    }

                
                   SchedulingDecision::Run{pid: process_pid,timeslice:self.timeslice}
                    
                }
            }
            else{
                SchedulingDecision::Done
            }
    }
    fn stop(&mut self, reason: StopReason) -> SyscallResult {
        
        match reason {
            StopReason::Syscall { syscall, remaining } => {
                    
               
                match syscall{
                Syscall::Fork(priority) =>{

                    let new_pid = self.next_pid;
                    self.next_pid += 1;
                    let mut new_process = MyProcess::new(new_pid);
                    new_process.priority = priority;
                    if self.next_pid!=2{
                        self.syncronize_timings(self.timeslice.get() - remaining-1);
                         
                         new_process.state = OtherProcessState::Ready;
                        if let Some(process) = self.processes.get_mut(0){
                            process.timings.1+=1;
                            self.increment_timings();

                             if remaining>0{
                                self.timeslice = NonZeroUsize::new(remaining).unwrap();
                                self.processes.push_back(new_process.clone());

                            }
                            else if self.processes.len()>1 {
                                let mut value = self.processes.pop_front().unwrap();
                                let mut value2 = self.processes.pop_front().unwrap();
                                value2.state = OtherProcessState::Running;
                                value.state = OtherProcessState::Ready;
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();

                                self.processes.push_front(value2);
                                self.processes.push_back(new_process.clone());

                                self.processes.push_back(value);

                            }
                                  
                          
                        }

                    }
                    else{
                        self.processes.push_back(new_process.clone());

                        self.running_process = new_process.clone();
                    }             
                    return SyscallResult::Pid(Pid::new(new_pid));
                },
                Syscall::Sleep(amount) =>{

                 
                    let _size = self.processes.len();
                    self.syncronize_timings(self.timeslice.get() - remaining-1);
                    let mut index2=0;
                    let mut final_index = 0;
                    for p in &self.processes {
                        if p.state == OtherProcessState::Running{
                            final_index = index2;
                        }
                        index2+=1;
                    }
                    if let Some(process) = self.processes.get_mut(final_index){
                        process.state = OtherProcessState::Waiting{event: None};
                        process.sleep_time = amount;
                        process.is_sleeping = true;
                        
                        if process.is_sleeping{
                            process.timings.1+=1;

                        }
                       
                        self.increment_timings();
                        if self.processes.len()!=1{
                             self.timeslice = NonZeroUsize::new(self.timeslice.get()-1).unwrap();
                        }
                    }
                 
                    return SyscallResult::Success;
                },
                Syscall::Exit=> {
                    self.syncronize_timings(self.timeslice.get() - remaining);
                   
                    if self.processes.len()!=1{
                        self.processes.pop_front();
                
                        if self.check_number_1_pid(){
                            if let Some(mut process2) = self.find_ready_process(){
                            
                                process2.state = OtherProcessState::Running;
                                if self.initial_timeslice>0 {
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                                }
                                for p in &mut self.processes {
                                    if p.pid()==process2.pid() {
                                        p.state = OtherProcessState::Running;
                                    }
                                }
                            }
                        }
                       
                        return SyscallResult::Success;
                        
                    }
                    else{
                        if !self.processes.is_empty(){
                        let mut process = self.processes.pop_front().unwrap();
                        process.timings.0+=1;
                        println!("{} {}",process.timings.0,self.processes.len());
                        }
                        
                        return SyscallResult::Success;
                    }
                },
                Syscall::Wait(event) => {
                   
                    if self.processes.len()==0{
                        return SyscallResult::Success;
                    }
                    let process = self.processes.get_mut(0).unwrap();
                    process.state = OtherProcessState::Waiting{event: Some(event)};
                    process.timings.1+=1;
                    process.timings.2+= self.timeslice.get() - remaining-1;
                    process.event_number=event;

                    for p in &mut self.processes {
                        if( p.state == OtherProcessState::Waiting{event: None}){
                            p.sleep_time-=self.timeslice.get()-remaining;
                         }
                         p.timings.0+= self.timeslice.get()-remaining;
                       
                    }
                    //Unwrap returneaza valoarea Some or None
                    if let Some(mut process2) = self.find_ready_process(){
                                        
                
                    
                        process2.state = OtherProcessState::Running;
                        let mut element: VecDeque<MyProcess> = self.processes.clone();
                        for p in &mut element {
                            
                            self.processes.pop_front();
                            if p.pid()==process2.pid(){
                                p.state = OtherProcessState::Running;
                            }
                            else{
                                self.processes.push_back(p.clone());
                            }

                        }
                        self.processes.push_front(process2.clone());
                        if self.initial_timeslice>0{
                        self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                        }
                    }
                    else if self.timeslice.get()-1>0{
                        self.timeslice = NonZeroUsize::new(self.timeslice.get()-1).unwrap();

                    }
                   

                    return SyscallResult::Success;
                },
                Syscall::Signal(event)=>{
                    let len = self.processes.len();
                    for i in 0..len {
                        for j in 0..(len - i-1) {
                            if self.processes[j].pid > self.processes[j + 1].pid 
                            && self.processes[j].state == (OtherProcessState::Waiting{event: Some(event)})
                            && self.processes[j+1].state == (OtherProcessState::Waiting{event: Some(event)}) {
                                self.processes.swap(j, j + 1);
                            }
                        }
                    }

                    let mut element =0;
                    let mut status = false;
                    let mut processes = self.processes.clone();
                    let mut processes_signal: VecDeque<MyProcess> = VecDeque::new();
                   
                    self.processes.clear();
                    for process in &mut processes {
                        
                        process.timings.0+= self.timeslice.get() - remaining;

                        if process.event_number==event{
                            process.state = OtherProcessState::Ready;
                            element+=1;
                            processes_signal.push_back(process.clone());


                        }
                        else{
                            if process.state == OtherProcessState::Running{
                                
                                process.timings.1+=1;
                                process.timings.2+= self.timeslice.get() - remaining-1;
                            }
                            if !status{
                                self.processes.push_front(process.clone());
                                status = true;
                            }
                            else{
                                self.processes.push_back(process.clone());
                            }
                            
                         }
                    }
                    
                  
                    for process in &mut processes_signal {
                        self.processes.push_back(process.clone());
                    }
                  
                
                    if element==0{                    
                        self.remaining_time=3;
                        if remaining!=0{
                            self.timeslice = NonZeroUsize::new(remaining).unwrap();
                        }
                        else if self.timeslice.get()-remaining>0 {
                            self.timeslice = NonZeroUsize::new(self.timeslice.get()-remaining).unwrap();
                        }
                    
                        
                    }
                    else if remaining!=0{
                            self.timeslice = NonZeroUsize::new(remaining).unwrap();
                    }
                    else if self.timeslice.get()-1!=0{
                                self.timeslice = NonZeroUsize::new(self.timeslice.get()-1).unwrap();
                    }
                    else if self.processes.len()>1{
                                let mut running_process = self.processes.pop_front().unwrap();
                                running_process.state = OtherProcessState::Ready;
                                self.processes.push_back(running_process.clone());
                                if !self.processes.is_empty(){
                                let mut process = self.processes.pop_front().unwrap();
                                process.state = OtherProcessState::Running;
                                self.processes.push_front(process.clone());
                                }
                               
                                if self.initial_timeslice>0 {
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                                }
                    }
                        
                                        

                    return SyscallResult::Success;

                },
                _ => {
                    self.syncronize_timings(self.timeslice.get() - remaining);
                    if remaining >= self.minimum_remaining_timeslice {
                        
                        if let Some(mut process) = self.processes.pop_front() {
                            let process_pid = process.pid();
                            self.remaining_time-=self.minimum_remaining_timeslice;
                            if(process.state == OtherProcessState::Waiting{event: None}) && self.processes.is_empty(){
                                self.remaining_time = self.initial_timeslice;
                                process.state = OtherProcessState::Running;
                            }
                           

                            self.processes.push_back(process);

                            return SyscallResult::Pid(process_pid);
                        }
                        else{
                            let new_pid = self.next_pid;
                            self.next_pid += 1;
                            self.remaining_time-=self.minimum_remaining_timeslice;
                        

                            let new_process = MyProcess::new(new_pid);
                            self.processes.push_back(new_process);

                            return SyscallResult::Pid(Pid::new(new_pid));
                        }
                    }
                    else if self.processes.is_empty(){
                        if let Some(process) = self.processes.pop_front() {
                            self.processes.push_back(process);
                        }
                        else{
                            let new_pid = self.next_pid;
                            self.next_pid += 1;

                            let new_process = MyProcess::new(new_pid);
                            self.processes.push_back(new_process);

                            return SyscallResult::Pid(Pid::new(new_pid));
                        }

                    }
                },
            }
             SyscallResult::Success
        },
            StopReason::Expired => {
               
                self.syncronize_timings(self.timeslice.get());

                if let Some(mut process) = self.processes.pop_front() {
                    for p in &mut self.processes {
                        if(p.sleep_time==0 && p.state == OtherProcessState::Waiting{event: None}){
                            p.state = OtherProcessState::Ready;
                        }
                    }
                    println!("{} {}", <NonZeroUsize as Into<usize>>::into(self.timeslice), self.minimum_remaining_timeslice);
                    if(process.state == OtherProcessState::Waiting{event: None}){
                        self.remaining_time = self.initial_timeslice;
                        process.state = OtherProcessState::Running;
                    }
                    process.state = OtherProcessState::Ready;
                    
                    if let Some(mut process2) = self.find_ready_process(){
                                        
                        process2.state = OtherProcessState::Running;
                        self.remaining_time=self.initial_timeslice;
                        if let Some(new_timeslice) = NonZeroUsize::new(self.timeslice.get() - 1) {
                            self.timeslice = new_timeslice;
                        }
                    
                        process2.state = OtherProcessState::Running;
                        for p in &mut self.processes {
                            if p.pid()==process2.pid(){
                                p.state = OtherProcessState::Running;
                            }
                        }
                    }
                    if self.initial_timeslice>0{
                    self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                    }
                        
                    self.processes.push_back(process);
                    
                }
                else{
                    let new_pid = self.next_pid;
                    let new_process = MyProcess::new(new_pid);
                    

                    self.processes.push_back(new_process);

                    return SyscallResult::Pid(Pid::new(new_pid));
                }
                SyscallResult::Success
            }
        }
    }

    fn list(&mut self) -> Vec<&dyn crate::scheduler::Process> {
       let mut vec = Vec::new();
       for p in &self.processes {
           let process = p as &dyn Process;
           vec.push(process);
       }
       vec
    
    }
}