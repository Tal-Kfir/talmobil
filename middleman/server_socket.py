import socket


class server():
    """
    
    Server thread main handler
    
    """
    
    def __init__(self, timeout):
        self.sock = socket.socket()
        self.sock.settimeout(timeout)
        self.timeout = timeout
        
    def bind(self, ip,port):
        try:
            self.sock.bind((ip,port))
            self.sock.listen()
            return (True, "Bind successfuly!")
        except Exception as e:
            return (False,e)
            
                

    def accept(self):
        return self.sock.accept()
    
    def close(self):
        self.sock.close()
    
    def __str__(self):
        return str(self.sock)
        
            