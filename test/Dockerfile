
FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

RUN microdnf -y install nginx

RUN mkdir /public

COPY nginx.conf /etc/nginx/nginx.conf
COPY index.html /public

CMD ["/usr/sbin/nginx", "-g", "daemon off;" ]

EXPOSE 8080
