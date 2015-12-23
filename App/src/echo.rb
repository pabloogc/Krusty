require 'socket'
server = TCPServer.new(12321)

while (connection = server.accept)
  Thread.new(connection) do |conn|
    port, host = conn.peeraddr[1,2]
    client = "#{host}:#{port}"
    puts "=============\r\n"
    begin
      loop do
        line = conn.readline
        puts "#{client} #{line}"
      end
    rescue EOFError
      conn.close
      puts "=============\r\n"
    end
  end
end
