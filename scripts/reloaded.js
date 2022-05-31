   function sleep(milliseconds) {  
      return new Promise(resolve => setTimeout(resolve, milliseconds));  
   }  
   async function rel() {         
	  await sleep(500);  
      location.reload();
   } 