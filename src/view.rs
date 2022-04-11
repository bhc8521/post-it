use crate::*;

#[near_bindgen]
impl PostIt {
    pub fn get_receive_posts_by_account(&self, account_id: PosterId) -> Vec<Post>{
        let account = self.accounts.get(&account_id).unwrap();
        let receive = account.receive;
        let mut posts: Vec<Post> = Vec::new();
        for hash in receive.iter() {
            posts.push(self.posts.get(&hash).unwrap());
        }
        posts
    }

    pub fn get_send_posts_by_account(&self, account_id: PosterId) -> Vec<Post> {
        let account = self.accounts.get(&account_id).unwrap();
        let send = account.send;
        let mut posts: Vec<Post> = Vec::new();
        for hash in send.iter() {
            posts.push(self.posts.get(&hash).unwrap());
        }
        posts
    }

    pub fn get_public_posts(&self) -> Vec<PublicPost> {
        self.public_posts.to_vec()
    }
}