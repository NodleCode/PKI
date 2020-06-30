FROM node:13

WORKDIR /usr/src/app
COPY package*.json ./

RUN yarn

COPY . .
RUN yarn bootstrap

ENTRYPOINT ["yarn"]