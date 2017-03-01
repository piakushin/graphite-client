const MAX_MSG_PATH: usize = 100;
const MAX_MSG_LEN_PLAIN: usize = 130;

struct GraphiteSender {
    stream: TcpStream,
}

impl GraphiteSender {
    fn new(host_addr: SocketAddr) -> GraphiteSender {
        GraphiteSender {
            stream: TcpStream::connect(host_addr).unwrap();
        }
    }
}

static int n = 0;
static int sockfd = 0;
static struct sockaddr_in serv_addr;

int graphite_init(const char *host, int port)
{
	if((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0)
	{
		log_error("socket");
		return 1;
	}

	int optval = 1; //-->TRUE
	int optlen = sizeof(optval);

	setsockopt(sockfd, SOL_SOCKET, SO_KEEPALIVE, &optval, optlen);

	struct timeval tv;

	tv.tv_sec = 30;  // 30 Secs Timeout
	tv.tv_usec = 0;  // Not init'ing this can cause strange errors

	setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, (char *)&tv,sizeof(struct timeval));

	memset(&serv_addr, '0', sizeof(serv_addr));

	serv_addr.sin_family = AF_INET;
	serv_addr.sin_port = htons(port);

	struct hostent *he = gethostbyname(host);

	if(!he)
	{
		log_error("can't find host name");
		return 1;
	}
	else
	{
		memcpy(&serv_addr.sin_addr,he->h_addr_list[0], he->h_length);
	}

	if( connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0)
	{
		log_error("Graphite connect failed");
		return 1;
	}

	return 0;
}

void graphite_finalize()
{
	if (sockfd != -1)
	{
		close(sockfd);
		sockfd = -1;
	}
}

int graphite_send(const char *message)
{
	n = send(sockfd, message, strlen(message), MSG_NOSIGNAL);
	if (n < 0)
	{
		log_error("Graphite send message failed");
		return 1;
	}
	return 0;
}

int graphite_send_plain( const char* path, float value, unsigned long timestamp )
{
	char spath[MAX_MSG_PATH];
	char message[MAX_MSG_LEN_PLAIN]; /* size = path + (value + timestamp) */

	/* make sure that path has a restricted length so it does not push the value + timestamp out of the message */
	snprintf( spath, MAX_MSG_PATH, "%s", path);

	/* format message as: <metric path> <metric value> <metric timestamp> */
	snprintf( message, MAX_MSG_LEN_PLAIN, "%s %.2f %lu\n", spath, value, timestamp );

	/* send to message to graphite */
	return graphite_send(message) != 0;
}
