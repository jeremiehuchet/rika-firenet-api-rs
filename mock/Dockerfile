FROM node:22@sha256:b501c082306a4f528bc4038cbf2fbb58095d583d0419a259b2114b5ac53d12e9
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
