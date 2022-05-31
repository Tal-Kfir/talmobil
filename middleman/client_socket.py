import socket
from ssl import SSLContext, PROTOCOL_TLS_CLIENT, wrap_socket
context = SSLContext(PROTOCOL_TLS_CLIENT)
context.load_default_certs()


class client():
    """
    
    Client thread main handler
    
    """
    
    def __init__(self, client, addr, timeout):
        self.sock = client
        self.sock.settimeout(timeout)
        
    def receive(self, buffer):
        try:
            return self.sock.recv(buffer)
        except socket.timeout as e:
            return ""
        except Exception as e:
            return None
            
    def send_redirect(self, IP):
        TARGET = IP[0]
        ADDON = "https://" if IP[1] else "http://"
        TARGET = (ADDON+TARGET).encode()
        try:
            msg = (b"HTTP/1.1 303 See Other\n" \
                           b"Location: " + TARGET + b"/\n\n" \
                           b"<html><body>Encryption Required.  Please go to <a href='" + TARGET + b"'>" + TARGET + b"</a> for this service.</body></html>\n\n")
            
            self.sock.send(msg)
        except Exception as e:
            print(e)
            
    def send_unavailable(self):
        try:
            self.sock.send(b"HTTP/1.1 503 Service Unavailable\n\n<html><body>Service unavailable, please try again later.</body></html>\n\n")
        except Exception as e:
            print(e)

    def close(self):
        try:
            self.sock.close()
        except Exception as e:
            pass
    
    def __str__(self):
        return "Client:\t" + str(self.sock) + "\n"
    
    def loop(self, IPS):
        
        for IP in IPS:
            ip, port = IP
            port = 443 if port else 80


            if port == 443:
                try:
                    with socket.create_connection((ip, port), timeout=1) as client:
                        with wrap_socket(client, ciphers="AES128-SHA256") as tls:
                            tls.close()
                        client.close()
                    return IP
                except Exception as e:
                    return
            try:
                temp = socket.socket()
                temp.settimeout(self.timeout)
                temp.connect((ip,port))
                if temp:
                    return IP
            except Exception as e:
                pass

def loop(user, domains, BUFFER):
    user.receive(BUFFER)
    msg = user.loop(domains)
    if msg:
        print("Server is on, redirecting")
        user.send_redirect(msg)
    else:
        print("Server is off, closing")
        user.send_unavailable()
            
    user.close()
    
            