var searchIndex = {};
searchIndex['demo_client'] = {"items":[],"paths":[]};
searchIndex['demo_server'] = {"items":[],"paths":[]};

searchIndex['string_telephone'] = {"items":[[0,"","string_telephone","\n# String Telephone"],[0,"packet","",""],[1,"Packet","string_telephone::packet","The underlying shape for transferring data."],[11,"protocol_id","","",0],[11,"sequence_id","","The current id of the packet",0],[11,"packet_type","","",0],[11,"packet_content","","Serialized user data goes in here",0],[2,"PacketType","","Headers for various different built-in message types"],[12,"Connect","","",1],[12,"Accept","","",1],[12,"Reject","","",1],[12,"Disconnect","","",1],[12,"Message","","",1],[2,"TaskCommand","","Commands to send to subprocesses"],[12,"Disconnect","","",2],[10,"eq","","",1],[10,"ne","","",1],[10,"fmt","","",1],[10,"clone","","",1],[10,"from_i64","","",1],[10,"from_u64","","",1],[10,"clone","","",0],[10,"connect","","",0],[10,"disconnect","","",0],[10,"accept","","",0],[10,"reject","","",0],[10,"message","","",0],[10,"deserialize","","",0],[10,"serialize","","",0],[0,"shared","string_telephone",""],[1,"ConnectionConfig","string_telephone::shared","General configuration for a connection"],[11,"protocol_id","","A shared ID to identify whether a connection should be accepted",3],[11,"timeout_period","","How long we should wait before hanging up",3],[11,"packet_deserializer","","A function to turn raw data into our packet format",3],[11,"packet_serializer","","A function to turn a packet into raw data",3],[1,"SequenceManager","","A helper struct to maintain packet ordering and acks"],[11,"last_sent_sequence_id","","",4],[11,"last_received_sequence_id","","",4],[10,"new","","Create a new ConnectionConfig object",3],[10,"clone","","",4],[10,"new","","Create a new SequenceManager",4],[10,"next_sequence_id","","Generate a new sequence ID for us",4],[10,"packet_is_newer","","Is a packet classed as newer than the last we received?",4],[10,"set_newest_packet","","Set the last packet we received",4],[0,"client","string_telephone",""],[1,"Client","string_telephone::client","Clientside implementation of UDP networking"],[11,"addr","","The socket we should use locally",5],[11,"target_addr","","The socket of the server we intent to connect to",5],[11,"config","","Basic configuration for connecting",5],[11,"connection_state","","What's the current state of our connection",5],[1,"ClientConnectionConfig","","Additional configuration options for a Client connection"],[11,"max_connect_retries","","How many times should we ask for a connection before giving up?",6],[11,"connect_attempt_timeout","","How long should each connection request await an answer?",6],[2,"ConnectionState","","The current state of a connection"],[12,"Disconnected","","",7],[12,"Connecting","","",7],[12,"Connected","","",7],[2,"PollFailResult","","A Poll attempt failed for some reason"],[12,"Empty","","",8],[12,"Disconnected","","",8],[10,"fmt","","",8],[10,"new","","Create a new ClientConnectionConfig object",6],[10,"connect","","Connect our Client to a target Server.\nWill block until either a valid connection is made, or we give up",5],[10,"poll","","Pop the last event off of our comms queue, if any",5],[10,"send","","Send a packet to the server",5],[10,"drop","","",5],[0,"server","string_telephone",""],[1,"Server","string_telephone::server","A UDP server, which manages multiple clients"],[11,"addr","","Which address to listen on",9],[11,"config","","Basic configuration for the server",9],[2,"PacketOrCommand","","Types of packet we can receive as a server"],[12,"UserPacket","","A message packet, containing whichever type we're set up to handle",10],[12,"Command","","An internal control packet",10],[10,"new","","Start listening on a given socket",9],[10,"poll","","Pump any messages that have been sent to us",9],[10,"cull","","Disconnect, and return, any sockets that have not contacted us for our timeout duration",9],[10,"send_to","","Send a packet to a specific address",9],[10,"send_to_many","","Send a packet to multiple addresses",9],[10,"send_to_all","","Send a packet to every connected client",9],[10,"all_connections","","List all of our current connections",9],[10,"drop","","",9]],"paths":[[1,"Packet"],[2,"PacketType"],[2,"TaskCommand"],[1,"ConnectionConfig"],[1,"SequenceManager"],[1,"Client"],[1,"ClientConnectionConfig"],[2,"ConnectionState"],[2,"PollFailResult"],[1,"Server"],[2,"PacketOrCommand"]]};

searchIndex['time'] = {"items":[[0,"","time","Simple time handling."],[1,"Timespec","","A record specifying a time value in seconds and nanoseconds."],[11,"sec","","",0],[11,"nsec","","",0],[1,"Tm","","Holds a calendar date and time broken down into its components (year, month, day, and so on),\nalso called a broken-down time value."],[11,"tm_sec","","Seconds after the minute - [0, 60]",1],[11,"tm_min","","Minutes after the hour - [0, 59]",1],[11,"tm_hour","","Hours after midnight - [0, 23]",1],[11,"tm_mday","","Day of the month - [1, 31]",1],[11,"tm_mon","","Months since January - [0, 11]",1],[11,"tm_year","","Years since 1900",1],[11,"tm_wday","","Days since Sunday - [0, 6]. 0 = Sunday, 1 = Monday, ..., 6 = Saturday.",1],[11,"tm_yday","","Days since January 1 - [0, 365]",1],[11,"tm_isdst","","Daylight Saving Time flag.",1],[11,"tm_utcoff","","Identifies the time zone that was used to compute this broken-down time value, including any\nadjustment for Daylight Saving Time. This is the number of seconds east of UTC. For example,\nfor U.S. Pacific Daylight Time, the value is -7*60*60 = -25200.",1],[11,"tm_nsec","","Nanoseconds after the second - [0, 10<sup>9</sup> - 1]",1],[1,"TmFmt","","A wrapper around a `Tm` and format string that implements Show."],[2,"ParseError","",""],[12,"InvalidSecond","","",2],[12,"InvalidMinute","","",2],[12,"InvalidHour","","",2],[12,"InvalidDay","","",2],[12,"InvalidMonth","","",2],[12,"InvalidYear","","",2],[12,"InvalidDayOfWeek","","",2],[12,"InvalidDayOfMonth","","",2],[12,"InvalidDayOfYear","","",2],[12,"InvalidZoneOffset","","",2],[12,"InvalidTime","","",2],[12,"MissingFormatConverter","","",2],[12,"InvalidFormatSpecifier","","",2],[12,"UnexpectedCharacter","","",2],[3,"get_time","","Returns the current time as a `timespec` containing the seconds and\nnanoseconds since 1970-01-01T00:00:00Z."],[3,"precise_time_ns","","Returns the current value of a high-resolution performance counter\nin nanoseconds since an unspecified epoch."],[3,"precise_time_s","","Returns the current value of a high-resolution performance counter\nin seconds since an unspecified epoch."],[3,"tzset","",""],[3,"empty_tm","",""],[3,"at_utc","","Returns the specified time in UTC"],[3,"now_utc","","Returns the current time in UTC"],[3,"at","","Returns the specified time in the local timezone"],[3,"now","","Returns the current time in the local timezone"],[3,"strptime","","Parses the time from the string according to the format string."],[3,"strftime","","Formats the time according to the format string."],[10,"fmt","","",0],[10,"decode","","",0],[10,"encode","","",0],[10,"cmp","","",0],[10,"partial_cmp","","",0],[10,"lt","","",0],[10,"le","","",0],[10,"gt","","",0],[10,"ge","","",0],[10,"eq","","",0],[10,"ne","","",0],[10,"clone","","",0],[10,"new","","",0],[10,"add","","",0],[10,"sub","","",0],[10,"fmt","","",1],[10,"eq","","",1],[10,"ne","","",1],[10,"clone","","",1],[10,"to_timespec","","Convert time to the seconds from January 1, 1970",1],[10,"to_local","","Convert time to the local timezone",1],[10,"to_utc","","Convert time to the UTC",1],[10,"ctime","","Returns a TmFmt that outputs according to the `asctime` format in ISO\nC, in the local timezone.",1],[10,"asctime","","Returns a TmFmt that outputs according to the `asctime` format in ISO\nC.",1],[10,"strftime","","Formats the time according to the format string.",1],[10,"rfc822","","Returns a TmFmt that outputs according to RFC 822.",1],[10,"rfc822z","","Returns a TmFmt that outputs according to RFC 822 with Zulu time.",1],[10,"rfc3339","","Returns a TmFmt that outputs according to RFC 3339. RFC 3339 is\ncompatible with ISO 8601.",1],[10,"eq","","",2],[10,"ne","","",2],[10,"fmt","","",2],[10,"fmt","","",3]],"paths":[[1,"Timespec"],[1,"Tm"],[2,"ParseError"],[1,"TmFmt"]]};

searchIndex['gcc'] = {"items":[[0,"","gcc",""],[1,"Config","","Extra configuration to pass to gcc."],[11,"include_directories","","Directories where gcc will look for header files.",0],[11,"definitions","","Additional definitions (`-DKEY` or `-DKEY=VALUE`).",0],[11,"objects","","Additional object files to link into the final archive",0],[3,"compile_library","","Compile a library from the given set of input C files."],[10,"default","","",0]],"paths":[[1,"Config"]]};

initSearch(searchIndex);
