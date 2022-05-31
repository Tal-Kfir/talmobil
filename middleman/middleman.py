from client_socket  import client, loop
from server_socket import server
from socket import timeout
from threading import Thread
from sys import argv

IP = "0.0.0.0"
PORT = 80

# Potential IPS and domains 
SOCKET_TIMEOUT = 0.1
BUFFER = 2048


def main():
    try:
        print(argv)
        POTENTIAL_DOMAINS = [(argv[1], argv[2]=='True')]
    except:
        print("Wrong input: py {file}.py {ip} {IsHTTPS}")
        return
    print(POTENTIAL_DOMAINS)
    sock = server(SOCKET_TIMEOUT)
    result = sock.bind(IP, PORT)
    if not result[0]:
        print(f"Couldn't initiate server, {result[1]}")
        return
    
    user = None
    thread_list = []
    
    while True:
        try:
            user = client(*sock.accept(), SOCKET_TIMEOUT)
            
            
            t = Thread(target = loop, args=(user,POTENTIAL_DOMAINS,BUFFER,))
            t.start()
            thread_list.append(t)
            
            user = None
        except timeout as e:
            if user:
                user.close()
        except Exception as e:
            print(e)
            break
        
        copy = thread_list.copy()
        for t in thread_list:
            if t.is_alive():
                continue
            copy.remove(t)
        thread_list = copy
        
    sock.close()


if __name__ == "__main__":
    main()