mainObjective: determine whether the styles suit the user

objectives {
  - Being able to upload a range of clothes, and each gets vectorized 
  - Being able to update the user picture and gets vectorized
  - Calculate the similarity between the clothes and the user image to determien whether they suit each other
  - Implement a simple in-memory vector search engine. 
}

objective UploadClothes {
  function upload(image: DynamicImage) -> String {
    - get the uploaded image
    - pass into the vectorization method
    - get the vectors
    - save the vectors into the vector database
    
    return the id
  };
  
  function delete(id: String) -> bool {};
  
  function get() -> Vec<DynamicImage> {};
}

objective UploadUserImage {
  function upload(image: DynamicImage) -> String {};
  
  function delete(id: String) -> bool {};
  
  function get() -> Vec<DynamicImage> {};
}

objective Calculate {
  function calculate_similarity(clothes_id: String, user_id: String) -> f64 {};
}