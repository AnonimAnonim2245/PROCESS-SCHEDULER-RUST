use std::collections::VecDeque;
use std::num::NonZeroUsize;
use crate::scheduler::Process;
use crate::scheduler::Syscall;
use crate::scheduler::MyProcess;
use crate::{Scheduler, SchedulingDecision, Pid, SyscallResult, StopReason};
use crate::ProcessState as OtherProcessState;
pub struct Cfs{
    processes: VecDeque<MyProcess>,
    running_process: MyProcess,
    timeslice: NonZeroUsize,
    initial_timeslice: usize,
    next_pid: usize,
    minimum_remaining_timeslice: usize,
    remaining_time : usize,

    
}

impl Cfs{
    fn find_smallest_vm_run(&mut self) -> usize{
        let mut min = 100000000;
        for process in &mut self.processes {
            if process.vmrun<min &&(process.state==OtherProcessState::Ready
                 || process.state==OtherProcessState::Running){
                min = process.vmrun;
            }
        }
        min

    }
    fn select_smallest_vm_run(&mut self, myprocess:MyProcess) -> Option<MyProcess>{
        let mut min = 100000000;
        let mut resultprocess = MyProcess::new(100);
        for process in &mut self.processes {
            if process.vmrun<min && myprocess.pid!=process.pid && (process.state==OtherProcessState::Ready
                 || process.state==OtherProcessState::Running){
                min = process.vmrun;
                resultprocess = process.clone();
            }
        }
        Some(resultprocess)
    

    }
    fn sizeofprocesses (&mut self) -> usize{
        let mut size = 0;
        for process in &mut self.processes {
            if process.state==OtherProcessState::Ready 
            || process.state==OtherProcessState::Running{
                size+=1;
            }
        }
        size
    }
    fn syncronize_timings(&mut self,timeslice : usize){
        for process2 in &mut self.processes {
            if process2.state == OtherProcessState::Running{
                process2.timings.0 += timeslice;
                process2.timings.2+= timeslice;
                process2.vmrun+=timeslice;
                process2.extra = format!("vruntime={}", process2.vmrun);
            }
            else{
                process2.timings.0 += timeslice;
                if(process2.state == OtherProcessState::Waiting{event: None}){
                    process2.sleep_time -= timeslice;
                }

            }
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
            if process.state == OtherProcessState::Ready {
                return Some(process.clone());
            }
        }
        None
    }
    fn find_running_process(&mut self) -> Option<MyProcess>{
        for process in &mut self.processes {
            if process.state == OtherProcessState::Running {
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
    pub fn new(timeslice: NonZeroUsize, minimum_remaining_timeslice: usize) -> Cfs {
        Cfs {
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
impl Scheduler for Cfs {
    
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
        && self.find_sleep_process().is_none() && !self.processes.is_empty(){
                            
                return SchedulingDecision::Deadlock;
        }
        if let Some(process) = self.find_running_process() {
                  
         SchedulingDecision::Run{pid: process.pid(),timeslice:self.timeslice}
                    
        }
        else if let Some(mut process) = self.processes.pop_front() {
                
            
                let pide = process.pid();
                
              
                if !process.is_sleeping && process.sleep_time!=0{
                    process.timings.0+=process.sleep_time;
                    self.syncronize_timings(process.sleep_time);
                    process.sleep_time=0;
                    if self.initial_timeslice>0{
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
                            if(self.initial_timeslice>0){
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
                                if process.sleep_time>0{
                                self.timeslice = std::num::NonZeroUsize::new(process.sleep_time).unwrap();
                                }
                                process.is_sleeping=false;
                                self.processes.push_front(process.clone());
                              
                                                    
                               
                                        
                            if process.sleep_time()>0{
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
                                    // nu e nevoie ca noi am verificat inainte cu Some si am dat push_back deci nu e gol
                                    process2 = self.processes.pop_front().unwrap();
                                }

                                if self.initial_timeslice>0{
                                     self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                                }
                                process2.state = OtherProcessState::Running;
                                self.processes.push_front(process2.clone());
                                
                                return SchedulingDecision::Run{pid: process2.pid(),timeslice:self.timeslice};

                            }
                         }
                         else{
                            
                            self.processes.push_back(process.clone());
                        
                            // Aici returneaza none sau some(process)
                            process = self.find_running_process().unwrap();
                            if self.initial_timeslice>0{
                            self.timeslice = std::num::NonZeroUsize::new(self.initial_timeslice).unwrap();
                            }

                            println!("{} {}",self.timeslice.get(),process.pid());
                            
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
            StopReason::Syscall { syscall, remaining  } => {
                    
               
                match syscall{
                Syscall::Fork(priority) =>{

                    let new_pid = self.next_pid;
                    self.next_pid += 1;
                    let mut new_process = MyProcess::new(new_pid);
                    new_process.priority = priority;
                    if self.next_pid!=2{
                          
                         new_process.state = OtherProcessState::Ready;
                         let mut index=0;
                         let mut index_lement=0;
                         for p in &mut self.processes {
                            
                            if p.state==OtherProcessState::Running{
                                index_lement=index;
                            }
                            index+=1;
                        }

                        if let Some(process) = self.processes.get_mut(index_lement){
                            process.timings.1+=1;
                            process.vmrun+=1;
                            process.extra = format!("vruntime={}", process.vmrun);


                            self.increment_timings();
                            new_process.vmrun=self.find_smallest_vm_run();
                             if remaining>0{
                                new_process.extra = format!("vruntime={}", new_process.vmrun);

                            
                                
                                self.processes.push_back(new_process.clone());

                                
                                if self.initial_timeslice/self.processes.len()>0{
                                   self.timeslice = NonZeroUsize::new(self.initial_timeslice/self.processes.len()).unwrap();
                                }


                            }
                            else if self.processes.len()>1{
                                let mut value = self.processes.pop_front().unwrap();
                                let mut value2 = self.processes.pop_front().unwrap();
                                value2.state = OtherProcessState::Running;
                                value.state = OtherProcessState::Ready;

                                self.processes.push_front(value2);
                                new_process.extra = format!("vruntime={}", new_process.vmrun);

                                self.processes.push_back(new_process.clone());

                                self.processes.push_back(value);
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice/self.sizeofprocesses()).unwrap();


                            }
                            if self.initial_timeslice/self.sizeofprocesses()>0{
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice/self.sizeofprocesses()).unwrap();
                            }

                        }

                    }
                    else{
                        new_process.vmrun=0;
                        new_process.extra = format!("vruntime={}", new_process.vmrun);                    
                        self.processes.push_back(new_process.clone());
                        if self.initial_timeslice/self.sizeofprocesses()>0{ 
                        self.timeslice = NonZeroUsize::new(self.initial_timeslice/self.sizeofprocesses()).unwrap();
                        }

                        self.running_process = new_process.clone();
                    }

                     
                    if self.initial_timeslice/self.sizeofprocesses()>0{
                      self.timeslice=NonZeroUsize::new(self.initial_timeslice/self.sizeofprocesses()).unwrap();
                    }

                   
                    return SyscallResult::Pid(Pid::new(new_pid));
                },
                Syscall::Sleep(amount) =>{

                    println!("Sleep");
                   
                    if let Some(process) = self.processes.get_mut(0){
                        process.vmrun+=1;
                        process.extra = format!("vruntime={}", process.vmrun);
                        process.state = OtherProcessState::Waiting{event: None};
                        process.sleep_time = amount;
                        process.is_sleeping = true;
                        if self.timeslice.get()-remaining!=1{
                            process.timings.0+= self.timeslice.get() - remaining-1;
                            process.vmrun+= self.timeslice.get() - remaining-1;
                            process.extra = format!("vruntime={}", process.vmrun);
                            process.timings.2+= self.timeslice.get() - remaining-1;

                        }
                        if process.is_sleeping{
                            process.timings.1+=1;

                        }
                       
                        self.increment_timings();
                        if self.timeslice.get()>0{
                          self.timeslice = NonZeroUsize::new(self.timeslice.get()).unwrap();
                        }
                    }
                   
                    return SyscallResult::Success;
                },
                Syscall::Exit=> {
                    println!("{} {} {}",self.timeslice.get(),remaining,self.processes.len());
                    
                        self.syncronize_timings( self.timeslice.get()-remaining); 

                    

               
                    // aici returneaza none sau some(process)
                    let process4 = self.find_running_process().unwrap();
                    println!("{}",process4.timings.0);
                    if self.processes.len()!=1{
                        let mut processes = self.processes.clone();
                        // in cazul acesta nu da panic, deoarece va rula pop numarul elememtelor din self.processes
                        for p in &mut processes{
                            let process = self.processes.pop_front().unwrap();
                            if p.state()!=OtherProcessState::Running{
                               self.processes.push_back(process.clone());
                            }
                        }
                       
                
                        if self.check_number_1_pid(){
                            if let Some(mut process2) = self.select_smallest_vm_run(process4){
                                if process2.state==OtherProcessState::Ready{
                                    process2.state = OtherProcessState::Running;
                                }
                                if self.initial_timeslice/2>0{
                                self.timeslice = NonZeroUsize::new(self.initial_timeslice/2).unwrap();
                                }
                                for p in &mut self.processes {
                                    if p.pid()==process2.pid() && p.state==OtherProcessState::Ready{
                                        p.state = OtherProcessState::Running;
                                    }
                                }
                            }
                        }
                      
                        if self.processes.len()==1 && self.initial_timeslice>0{
                            self.timeslice = NonZeroUsize::new(self.initial_timeslice).unwrap();
                        }
                        return SyscallResult::Success;
                        
                    }
                    else{
                        if !self.processes.is_empty(){
                         self.processes.pop_front().unwrap();
                        }
                    
                        return SyscallResult::Success;
                    }
                },
                Syscall::Wait(event) => {
                   
                  
                    let mut e=0;
                    let mut i =0;
                    for p in &mut self.processes {
                        e+=1;
                        if p.state==OtherProcessState::Running{
                            i=e-1;
                        }
                    }
                    // implicit i este mai mare decat zero iar i va fi mereu e-1 (iar e este mereu mai mare decat 0 si mai mic  sau egal decat lungimea vectorului)
                    // asadar i va fi mereu mai mic decat lungimea vectorului

                    let process = self.processes.get_mut(i).unwrap();
                    println!("{}",process.sleep_time());
                    process.state = OtherProcessState::Waiting{event: Some(event)};
                    process.timings.1+=1;
                    process.vmrun+=1;
                    process.extra = format!("vruntime={}", process.vmrun);
                    process.timings.2+= self.timeslice.get() - remaining-1;
                    process.vmrun += self.timeslice.get() - remaining-1;
                    process.extra = format!("vruntime={}", process.vmrun);

                    process.event_number=event;

                    for p in &mut self.processes {
                        if( p.state == OtherProcessState::Waiting{event: None}){
                            p.sleep_time-= self.timeslice.get()-remaining;
                         }
                         p.timings.0+= self.timeslice.get()-remaining;

                    }
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
                    else{
                        if self.timeslice.get()-1>0{  
                          self.timeslice = NonZeroUsize::new(self.timeslice.get()-1).unwrap();
                        }

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
                    for process in &mut self.processes {
                        if process.event_number==event{
                            process.state = OtherProcessState::Ready;

                        }
                        if process.state == OtherProcessState::Running{
                            process.timings.1+=1;
                            process.vmrun+=1;
                            process.vmrun+= self.timeslice.get()-remaining-1;
                            process.extra = format!("vruntime={}", process.vmrun);
                            process.timings.2+= self.timeslice.get() - remaining-1;
                        }
                        process.timings.0+= self.timeslice.get() - remaining;
                    }
                  
               
                    if self.initial_timeslice/self.processes.len()>0{
                         self.timeslice = NonZeroUsize::new(self.initial_timeslice/self.processes.len()).unwrap();
                    }
                    

                    return SyscallResult::Success;

                },
                _ => {
                    self.syncronize_timings(self.timeslice.get() - remaining);
 
                    if remaining >= self.minimum_remaining_timeslice {
                        
                        if let Some(mut process) = self.processes.pop_front() {
                            process.vmrun += self.timeslice.get() - remaining-1;
                            process.extra = format!("vruntime={}", process.vmrun);
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
                    let mut e=0;
                    let mut i =0;
                    let processes: VecDeque<MyProcess> = self.processes.clone();
                    // i este intre 0 si self.processes.len()-1 deci nu are cum
                    // sa dea panic la get_mut
                    for p in &mut self.processes {
                        e+=1;
                        if p.state==OtherProcessState::Running{
                            i=e-1;
                        }
                    }
                   
                    let process = self.processes.get_mut(i).unwrap();
                    
                    self.remaining_time = self.initial_timeslice;


                    if(process.state == OtherProcessState::Waiting{event: None}){
                        self.remaining_time = self.initial_timeslice;
                        process.state = OtherProcessState::Running;
                    }
                    process.state = OtherProcessState::Ready;
                  
                    for mut p in  processes {
                        if p.pid()==process.pid(){
                            p.state = OtherProcessState::Ready;
                        }
                    }
                    let myprocess = process.clone();
                    if let Some(mut process2) = self.select_smallest_vm_run(myprocess){
                                        
                        process2.state = OtherProcessState::Running;
                        self.remaining_time=self.initial_timeslice/2;
                        
                    
                        process2.state = OtherProcessState::Running;
                        for p in &mut self.processes {
                            if p.pid()==process2.pid(){
                                p.state = OtherProcessState::Running;
                            }
                        }

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